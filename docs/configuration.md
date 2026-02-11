---
layout: default
title: Vetis - Configuration
nav_order: 7
---

# Configuration

This document describes the configuration structure for the Vetis web server.

## Overview

The configuration file defines server listeners and virtual hosts for routing HTTP requests. The server supports multiple interfaces, SSL/TLS, and flexible static file serving.

## Configuration Structure

### Server Section

Defines the main server settings and listening interfaces.

```yaml
server:
  listeners:
    - interface: "0.0.0.0"
      port: 8080
      protocol: "Http1"
```

#### Listener Configuration

- **interface**: Network interface to bind to
  - `"0.0.0.0"` - Listen on all available network IPv4 interfaces
  - `"::"` - Listen on all available network IPv6 interfaces
  - `"127.0.0.1"` - Listen only on localhost
  - Specific IP address for targeted binding

- **port**: TCP port number for incoming connections
  - Range: 1-65535
  - Common ports: 80 (HTTP), 443 (HTTPS), 8080 (development)

- **protocol**: HTTP protocol version
  - `"Http1"` - HTTP/1.1 protocol
  - `"Http2"` - HTTP/2 protocol (if supported)
  - `"Http3"` - HTTP/3 protocol (if supported)

### Virtual Hosts Section

Defines virtual host configurations for handling different domains or paths.

```yaml
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

#### Virtual Host Configuration

- **hostname**: Domain name or hostname for this virtual host
  - `"localhost"` - Local development
  - `"example.com"` - Production domain
  - `"*.example.com"` - Wildcard subdomains

- **port**: Port number this virtual host responds on
  - Must match one of the configured listener ports

- **root_directory**: File system root directory for this virtual host
  - Absolute path to the document root
  - All file paths are resolved relative to this directory

- **enable_logging**: Enable request/response logging
  - `true` - Log all requests to stdout or file
  - `false` - Disable logging for performance

#### Error Pages Configuration

Custom error pages for different HTTP status codes:

```yaml
error_pages:
  404: "404.html"
  500: "500.html"
  403: "forbidden.html"
```

- **Key**: HTTP status code (404, 500, 403, etc.)
- **Value**: File name relative to the virtual host root directory

#### Static Paths Configuration

Defines URL patterns for serving static files:

```yaml
static_paths:
  - uri: "/"
    directory: "/home/rogerio/Documentos/Temp/vetis/static"
    extensions: "\\.(html)$"
    index_files:
      - "index.html"
```

- **uri**: URL path pattern to match
  - `"/"` - Root path and all subpaths
  - `"/static"` - Only paths starting with /static
  - `"/assets/*"` - Wildcard matching

- **directory**: File system directory containing static files
  - Can be absolute or relative to virtual host root

- **extensions**: Regular expression for allowed file extensions
  - `"\\.(html)$"` - Only HTML files
  - `"\\.(css|js|png|jpg|gif)$"` - Common web assets
  - `".*"` - All files

- **index_files**: List of default files to serve for directory requests
  - Served in order when requesting a directory URI
  - Common: `["index.html", "index.htm"]`

## Example Configurations

### Basic Development Server

```yaml
server:
  listeners:
    - interface: "127.0.0.1"
      port: 3000
      protocol: "Http1"

virtual_hosts:
  - hostname: "localhost"
    port: 3000
    root_directory: "./public"
    enable_logging: true
    static_paths:
      - uri: "/"
        directory: "./public"
        extensions: ".*"
        index_files:
          - "index.html"
```

### Production HTTPS Server

```yaml
server:
  listeners:
    - interface: "0.0.0.0"
      port: 443
      protocol: "Http2"

virtual_hosts:
  - hostname: "example.com"
    port: 443
    root_directory: "/var/www/example.com"
    enable_logging: true
    error_pages:
      404: "errors/404.html"
      500: "errors/500.html"
    static_paths:
      - uri: "/"
        directory: "/var/www/example.com/public"
        extensions: "\\.(html|css|js|png|jpg|gif|svg)$"
        index_files:
          - "index.html"
```

## Security Considerations

1. **Interface Binding**: Use `"127.0.0.1"` for development to prevent external access
2. **SSL Configuration**: Always enable SSL in production environments
3. **File Extensions**: Restrict file extensions to prevent serving sensitive files
4. **Directory Traversal**: Ensure directory paths are properly validated
5. **Logging**: Enable logging in production for security monitoring

## Performance Tips

1. **Disable Logging**: Set `enable_logging: false` in production for better performance
2. **HTTP/2**: Use `"Http2"` protocol for better multiplexing
3. **Static File Caching**: Configure appropriate cache headers for static assets
4. **File Extension Filtering**: Limit extensions to reduce unnecessary file system checks

## Troubleshooting

### Common Issues

- **Port Already in Use**: Change the port number or stop conflicting services
- **Permission Denied**: Ensure the server has read access to configured directories
- **404 Errors**: Check that `root_directory` and `static_paths` are correctly configured

### Debug Mode

Enable logging to troubleshoot routing and file serving issues:

```yaml
virtual_hosts:
  - hostname: "localhost"
    port: 8080
    root_directory: "./public"
    enable_logging: true
```
