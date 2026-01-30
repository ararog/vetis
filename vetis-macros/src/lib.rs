#[macro_export]
macro_rules! http {
    (hostname => $hostname:expr, port => $port:expr, interface => $interface:expr, handler => $handler:ident) => {
        async move {
            use vetis::{
                config::{ListenerConfig, ServerConfig, VirtualHostConfig},
                errors::VetisError,
                server::path::HandlerPath,
                server::virtual_host::VirtualHost,
                Vetis,
            };

            let listener = ListenerConfig::builder()
                .port($port)
                .interface($interface.to_string())
                .build();

            let config = ServerConfig::builder()
                .add_listener(listener)
                .build();

            let virtual_host_config = VirtualHostConfig::builder()
                .hostname($hostname)
                .port($port)
                .build()?;

            let mut virtual_host = VirtualHost::new(virtual_host_config);

            let root_path = HandlerPath::new_host_path("/".to_string(), Box::new($handler));

            virtual_host.add_path(root_path);

            let mut vetis = vetis::Vetis::new(config);

            vetis
                .add_virtual_host(virtual_host)
                .await;

            Ok::<Vetis, Box<VetisError>>(vetis)
        }
    };

    (hostname => $hostname:literal, port => $port:literal, interface => $interface:literal, handler => $handler:ident) => {
        async move {
            use vetis::{
                config::{ListenerConfig, ServerConfig, VirtualHostConfig},
                errors::VetisError,
                server::{path::HandlerPath, virtual_host::VirtualHost},
                Vetis,
            };

            let listener = ListenerConfig::builder()
                .port($port)
                .interface($interface.to_string())
                .build();

            let config = ServerConfig::builder()
                .add_listener(listener)
                .build();

            let virtual_host_config = VirtualHostConfig::builder()
                .hostname($hostname.to_string())
                .port($port)
                .build()?;

            let mut virtual_host = VirtualHost::new(virtual_host_config);

            let root_path = HandlerPath::new_host_path("/".to_string(), Box::new($handler));

            virtual_host.add_path(root_path);

            let mut vetis = Vetis::new(config);

            vetis
                .add_virtual_host(virtual_host)
                .await;

            Ok::<Vetis, Box<VetisError>>(vetis)
        }
    };
}

#[macro_export]
macro_rules! https {
    (hostname => &$hostname:ident, port => &$port:ident, interface => &$interface:ident, &cert => &$cert:ident, &key => &$key:ident) => {};
}
