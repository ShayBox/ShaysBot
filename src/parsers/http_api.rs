use std::{collections::VecDeque, io::Read, str::FromStr, sync::Mutex};

use azalea::{
    app::{App, Plugin, Startup, Update},
    ecs::prelude::*,
    local_player::TabList,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use tiny_http::{Header, Request, Response, Server};

use crate::prelude::*;

#[derive(Default, Resource)]
pub struct ApiServer(Option<Server>);

/// Local HTTP API command parsing integration
pub struct HttpApiParserPlugin;

impl Plugin for HttpApiParserPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ApiServer::default())
            .add_systems(Startup, Self::handle_startup)
            .add_systems(
                Update,
                (
                    Self::handle_api_requests
                        .before(MinecraftParserPlugin::handle_chat_received_events),
                    Self::handle_send_msg_events,
                ),
            );
    }
}

impl HttpApiParserPlugin {
    pub fn handle_startup(mut api_server: ResMut<ApiServer>, settings: Res<GlobalSettings>) {
        match Server::http(settings.http_api.bind_addr.clone()) {
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
        mut cmd_events: EventWriter<CmdEvent>,
        query: Query<Entity>,
        api_server: ResMut<ApiServer>,
        settings: Res<GlobalSettings>,
        tab_list: Res<TabList>,
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

        let Some(uuid) = tab_list
            .iter()
            .find(|(_, player_info)| player_info.profile.name == username)
            .map(|(uuid, _)| uuid)
        else {
            warn!("[API] {username} tried but isn't online");
            send_text(request, "User isn't online", 404);
            return;
        };

        let Some(user) = settings.users.get(uuid) else {
            warn!("[API] {username} tried but isn't whitelisted!");
            send_text(request, "User isn't whitelisted", 404);
            return;
        };

        if user.api_password.is_empty() || user.api_password != password {
            warn!("[API] {username} tried an incorrect password!");
            send_text(request, "Incorrect password", 401);
            return;
        }

        // TODO: Separate the rest into a another handle for routes.
        let url = request.url().replace("%20", " ");
        let Some(message) = url.strip_prefix("/cmd/") else {
            send_text(request, "Invalid route, available: /cmd/<command>", 500);
            return;
        };

        let mut args = message
            .split(' ')
            .map(String::from)
            .collect::<VecDeque<_>>();
        let Some(alias) = args.pop_front() else {
            return; /* Command Missing */
        };

        let Some(cmd) = Cmds::find(&alias.replace(&settings.command_prefix, "")) else {
            return; /* Command Invalid */
        };

        let mut cmd_event = CmdEvent {
            args: args.clone(),
            cmd,
            entity: None,
            message: false,
            sender: CmdSender::ApiServer(*uuid),
            source: CmdSource::ApiServer(Arc::new(Mutex::new(Some(request)))),
        };

        cmd_events.write_batch(std::iter::once(cmd_event.clone()).chain(query.iter().map(
            |entity| {
                cmd_event.entity = Some(entity);
                cmd_event.clone()
            },
        )));
    }

    pub fn handle_send_msg_events(mut msg_events: EventReader<MsgEvent>) {
        for event in msg_events.read().cloned() {
            #[rustfmt::skip]
            let (
                CmdSource::ApiServer(request),
                CmdSender::ApiServer(_)
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
