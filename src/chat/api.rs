use std::{collections::VecDeque, io::Read, str::FromStr, sync::Mutex};

use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity},
};
use base64::{prelude::BASE64_STANDARD, Engine};
use tiny_http::{Header, Request, Response, Server};

use crate::prelude::*;

#[derive(Default, Resource)]
pub struct ApiServer(Option<Server>);

pub struct ApiServerPlugin;

impl Plugin for ApiServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ApiServer::default())
            .add_systems(Startup, Self::handle_startup)
            .add_systems(
                Update,
                (Self::handle_api_requests, Self::handle_send_whisper_events),
            );
    }
}

impl ApiServerPlugin {
    pub fn handle_startup(mut api_server: ResMut<ApiServer>, settings: Res<GlobalSettings>) {
        match Server::http(settings.api_server.bind_addr.clone()) {
            Ok(server) => {
                info!("API Server @ {}", server.server_addr());
                api_server.0 = Some(server);
            }
            Err(error) => {
                error!("Failed to start API server: {error}");
            }
        }
    }

    /// # Panics
    /// Will panic if `Header::from_str` fails.
    pub fn handle_api_requests(
        mut command_events: EventWriter<CommandEvent>,
        mut query: Query<Entity, (With<Player>, With<LocalEntity>)>,
        api_server: ResMut<ApiServer>,
        settings: Res<GlobalSettings>,
    ) {
        let Some(server) = &api_server.0 else {
            error!("[API] Server not running.");
            return;
        };

        let Ok(Some(request)) = server.try_recv() else {
            return; /* No API Request */
        };

        let Some(header) = request
            .headers()
            .iter()
            .find(|header| header.field.equiv("Authorization"))
        else {
            let header = Header::from_str("WWW-Authenticate: Basic").unwrap();
            let response = Response::from_string("Unauthorized")
                .with_status_code(401)
                .with_header(header);

            send_response(request, response);
            return;
        };

        let encoded = header.value.as_str().replace("Basic ", "");
        let Ok(bytes) = BASE64_STANDARD.decode(&encoded) else {
            send_text(request, "Invalid BASE64", 406);
            return;
        };

        let Ok(credentials) = String::from_utf8(bytes) else {
            send_text(request, "Invalid UTF-8", 406);
            return;
        };

        // RFC 2617 provides support for passwords with colons
        let Some((username, password)) = credentials.split_once(':') else {
            send_text(request, "Invalid Format", 406);
            return;
        };

        if username != settings.api_server.username || password != settings.api_server.password {
            send_text(request, "Invalid Credentials", 406);
            return;
        }

        // TODO: Separate the rest into a another handle for routes.

        let url = request.url().replace("%20", " ");
        let Some(message) = url.strip_prefix("/cmd/") else {
            if let Err(error) = request.respond(
                Response::from_string("Invalid route, available: /cmd/<command>")
                    .with_status_code(500),
            ) {
                error!("[API] Failed to send response: {error}");
            };
            return; /* No command found */
        };

        let mut events = Vec::new();
        let request = Arc::new(Mutex::new(Some(request)));
        for entity in &mut query {
            let mut args = message
                .split(' ')
                .map(String::from)
                .collect::<VecDeque<_>>();
            let Some(alias) = args.pop_front() else {
                continue; /* Command Missing */
            };

            let Some(command) = ChatCmds::find(&alias.replace(&settings.command_prefix, "")) else {
                continue; /* Command Invalid */
            };

            events.push(CommandEvent {
                entity,
                args,
                command,
                message: false,
                source: CommandSource::ApiServer(request.clone()),
                sender: CommandSender::ApiServer,
            });
        }

        command_events.send_batch(events);
    }

    pub fn handle_send_whisper_events(mut whisper_events: EventReader<WhisperEvent>) {
        for event in whisper_events.read().cloned() {
            #[rustfmt::skip]
            let (
                CommandSource::ApiServer(request),
                CommandSender::ApiServer
            ) = (event.source, event.sender) else {
                continue;
            };

            info!("[API] [{}] {}", event.status, event.content);

            let Ok(mut request) = request.lock() else {
                continue; /* Locked */
            };

            let Some(request) = std::mem::take(&mut *request) else {
                continue; /* Taken */
            };

            let response = Response::from_string(event.content).with_status_code(event.status);
            if let Err(error) = request.respond(response) {
                error!("[API] Error sending response: {error}");
            }
        }
    }
}

pub fn send_text(request: Request, text: &str, code: u16) {
    let response = Response::from_string(text).with_status_code(code);
    send_response(request, response);
}

pub fn send_response<R: Read>(request: Request, response: Response<R>) {
    if let Err(error) = request.respond(response) {
        error!("[API] Error sending response: {error}");
    }
}
