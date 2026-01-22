# Generating self-signed certificates

## CA Certificate

```bash
openssl req -x509 -noenc -subj '/CN=My CA' -newkey rsa:4096 -keyout ca.key -out ca.crt -days 3650
```

### Export as DER:

```bash
openssl x509 -in ca.crt -out ca.der -outform DER
```

## Server 

### Private Key

```bash
openssl genrsa -out server.key 4096
```

### Certificate Request

```bash
openssl req -new -key server.key -out server.csr -subj '/CN=localhost'
```

### Export as DER:

```bash
openssl x509 -inform pem -in server.key -outform der -out server.der.key
```

### Create server.cnf

```bash
# server.cnf
[ req ]
prompt = no
distinguished_name = req_distinguished_name
req_extensions = req_ext

[ req_distinguished_name ]
CN = localhost

[ req_ext ]
subjectAltName = @alt_names

[ alt_names ]
DNS.1 = localhost
IP.1 = 127.0.0.1
# Add more DNS names or IPs as needed, e.g., DNS.2 = myapp.internal
```

### Sign the Certificate Request

```bash
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 3650 -extfile server.cnf -extensions req_ext
```

## Client

### Private Key and certificate request

```bash
openssl req -new -newkey rsa:4096 -nodes -keyout client.key -out client.csr -config client.cnf -sha256
```

### Sign the Certificate Request

```bash
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt -days 3650 -sha256 -extfile client.cnf
```

### Export as PKCS#8 DER

```bash
openssl pkcs8 -topk8 -inform PEM -outform DER -in client.key -out client.der.key -nocrypt
```

### Export as PKCS#12 (client.key and client.crt are PEM encoded)

```bash
openssl pkcs12 -export -out client.p12 -inkey client.key -in client.crt -name "Client Certificate"
```
