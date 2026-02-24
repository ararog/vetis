# Language Support

## Python

Either ASGI, WSGI or RSGI can be used. Only one of them should be enabled at a time.
All interfaces assume you have a Python interpreter available.

### ASGI

#### Setup

In order to enable ASGI support, you need to enable the `interface`, `python` and `asgi` features.

```toml
[dependencies]
vetis = { version = "0.1.0", features = ["interface", "python", "asgi"] }
```

### WSGI

#### Setup

In order to enable WSGI support, you need to enable the `interface`, `python` and `wsgi` features.

```toml
[dependencies]
vetis = { version = "0.1.0", features = ["interface", "python", "wsgi"] }
```

#### Notes

- To obtain maximum performance while running your application, please
  provide the following configuration on vetis:

```yaml
workers: 4
max_blocking_threads: 1
```

### RSGI

#### Setup

In order to enable RSGI support, you need to enable the `interface`, `python` and `rsgi` features.

```toml
[dependencies]
vetis = { version = "0.1.0", features = ["interface", "python", "rsgi"] }
```

## PHP

### Setup

You must build PHP as a static library, either manually or using SPC.

Building PHP with SPC on Linux:

```bash
curl -fsSL -o spc https://dl.static-php.dev/static-php-cli/spc-bin/nightly/spc-linux-x86_64
./spc doctor --auto-fix
./spc download php-src --with-php=8.3 --for-extensions="curl,dba,dom,exif,mysqli..."
./spc build curl,dba,dom,exif,mysqli... --build-embed
```

In order to enable PHP support, you need to enable the `interface` and `php` features.

```toml
[dependencies]
vetis = { version = "0.1.0", features = ["interface", "php"] }
```

## Ruby

### Setup

You must build Ruby as a static library.

Ruby requires the `clang` compiler to be installed.

```bash
sudo apt update
sudo apt install ruby-full
sudo apt-get install clang
```

After that, you can enable Ruby support.

In order to enable Ruby support, you need to enable the `interface` and `ruby` features.

```toml
[dependencies]
vetis = { version = "0.1.0", features = ["interface", "ruby"] }
```

This is the minimum required to enable Ruby support.