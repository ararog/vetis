---
layout: default
title: Vetis - Standalone Server
nav_order: 6
---

# Standalone Server

Vetis can be also used as a standalone server, we bundle a binary with the most common features enabled.

In order to start the server, run:

```bash
vetis
```

## Configuration

The server can be configured using environment variables or a configuration file.

### Example

```yaml
server:
  listeners:
    - interface: "0.0.0.0"
      port: 8080
      ssl: false
      protocol: "Http1"

virtual_hosts:
  - hostname: "localhost"
    port: 8080
    root_directory: "/home/rogerio/Documentos/Temp/vetis"
    enable_logging: false
    error_pages:
      404: "404.html"
    static_paths:
      - uri: "/"
        directory: "/home/rogerio/Documentos/Temp/vetis/static"
        extensions: "\\.(html)$"
        index_files:
          - "index.html"
```

Note that the configuration file is optional, and the server will use default values if not provided.

For more details, please refer to the [configuration documentation](configuration.md).