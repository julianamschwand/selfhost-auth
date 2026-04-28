# Selfhost Auth
Selfhost auth is a simple auth you can use to protect self hosted services on your webserver. 
It's especially useful to protect services that usually don't provide authentication themselves, like SearXNG for example.

<img width="2557" height="1440" alt="Selfhost Auth login page" src="https://github.com/user-attachments/assets/f225f01d-4606-4184-b2ba-4250c6230e0b" />

## How it works
Selfhost auth works by a web server first making a request to it to check if a client is logged in (with a session cookie). 
If the client is not logged in, instead of opening the protected site, 
it reroutes to selfhost auth where the client can authenticate and get a session cookie back if the login is successful. 

## Limitations
- No account system for multiple users
- Selfhost auth and the protected page must live under the same domain
- Proxy setup on the web server is manual (an example for nginx is provided in the setup instructions)

## Setup
### Docker / Podman
#### 1. Create a new directory for the service
```bash
mkdir selfhost-auth
cd selfhost-auth
```

#### 2. Create a docker compose file

docker-compose.yml:
```yaml
services:
  selfhost-auth:
    image: ghcr.io/julianamschwand/selfhost-auth
    ports:
      - 8080:8080
    volumes:
      - ./data:/app/data
    environment:
      - APP_ENV=production               # Remove if serving over http
      - PASSWORD_HASH=yourpasswordhash   # Replace with your own password hash
      - COOKIE_DOMAIN=example.com        # Replace with your domain name
    restart: unless-stopped 
```
Change the environment variables to your needs

#### 3. Hash your password

```bash
read -s PASSWORD && docker run --rm ghcr.io/julianamschwand/selfhost-auth ./selfhost-auth "$PASSWORD"
```
Then set the password hash in your docker-compose.yml

#### 4. Set up a reverse proxy (example for nginx)
```nginx
server {
  listen 443 ssl;
  server_name auth.example.com;
  ssl_certificate /path/to/ssl_cert.pem;
  ssl_certificate_key /path/to/ssl_cert_key.pem;

  location / {
    proxy_pass http://localhost:8080;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
  }
}
```

#### 5. Add auth check configuration for protected subdomains (example for nginx)

```nginx
server {
  listen 443 ssl;
  server_name searxng.example.com;
  ssl_certificate /path/to/ssl_cert.pem;
  ssl_certificate_key /path/to/ssl_cert_key.pem;

  location / {
    proxy_pass http://localhost:8888;

    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    auth_request /auth-check;
    auth_request_set $auth_status $upstream_status;
    error_page 401 = @login_redirect;
  }

  location = /auth-check {
    internal;
    proxy_pass http://localhost:8080/check-login; # selfhost auth (change port if needed)
    proxy_pass_request_body off;
    proxy_set_header Content-Length "";
    proxy_set_header X-Original-URI $request_uri;
    proxy_set_header X-Original-Host $host;
    proxy_set_header Cookie $http_cookie;
  }

  location @login_redirect {
    return 302 https://auth.example.com?redirect=$scheme://$host$request_uri;
  }
}
```

#### 6. Start Selfhost Auth
```bash
docker compose up -d
```

If you've set everything up correctly it should work now
