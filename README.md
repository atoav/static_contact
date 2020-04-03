# static_contact

static_contact is a server side tool that relays http POST requests with form fields like name, email, phone and message to email targets via SMTP.  
This is meant to be a simple self-hosted solution for contact forms on static websites. You add a form and a little bit of javascript to a static website, it POSTs the message to a different server, does a few checks and sends it to an target adress via email. It is possible to connect multiple static websites to one single instance of the service.

## Installation
static_contact runs as a systemd-service on a local port. Although it would be possible to expose this directly to the web, it is recommended to run this service behind a a reverse proxy server (like nginx).

1. Install Rust by using `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install cargo deb by running `cargo install cargo-deb`
3. Make sure to have `libssl-dev` and `pkg-config` installed
4. Get the package by cloning this git repository and `cd` into it
5. Compile via `cargo build --release`
5. Bundle debian package by running `cargo deb`
6. Install package with `sudo dpkg -i target/debian/*.deb`

## Configuration
Before running the thing you should update your configuration to make it work.
Do this by editing the config at `/etc/static_contact/config.toml` e.g. like this:
```bash
sudo vim /etc/static_contact/config.toml
```  

Pay special attention to the SMTP configuration, it should reflect the values of your SMTP server.

## Operation
Start the service via `sudo systemctl start static_contact`
Check the service status via `sudo systemctl status static_contact`
Check the logs via `journalctl -u static_contact`
Stop the service via `sudo systemctl stop static_contact`
Automatically start the service on boot via `sudo systemctl enable static_contact`  

## Example Nginx Configuration
Note that this used Certbot to generate Let's Encrypt certificates:

```
server {
        server_name     post.example.com;

        add_header      Cache-Control   no-cache;
        add_header Strict-Transport-Security "max-age=31536000; includeSubdomains; preload;";
        add_header x-frame-options SAMEORIGIN;
        add_header X-Content-Type-Options nosniff;
        add_header X-XSS-Protection "1; mode=block";

        # listen [::]:443 ssl ipv6only=on; # managed by Certbot
        listen 443 ssl; # managed by Certbot

        client_max_body_size 10M;
        client_body_buffer_size 128k;

        ssl_certificate /etc/letsencrypt/live/post.example.com/fullchain.pem; # managed by Certbot
        ssl_certificate_key /etc/letsencrypt/live/post.example.com/privkey.pem; # managed by Certbot
        include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot

        location / {
                proxy_set_header Host $host;
                proxy_set_header X-Forwarded-Host $host;
                proxy_set_header X-Forwarded-Server $host;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_http_version 1.1;
                proxy_set_header Upgrade $http_upgrade;
                proxy_set_header Connection "upgrade";

                proxy_redirect http://127.0.0.1:8088/ /;
                proxy_redirect ws://127.0.0.1:8088/api/socket /api/socket;
                proxy_pass http://127.0.0.1:8088;
                proxy_read_timeout 86400s;
                proxy_send_timeout 86400s;
                allow all; # Any IP can perform any other requests
        }
}

server {
    if ($host = post.example.com) {
        return 301 https://$host$request_uri;
    } # managed by Certbot

    listen          80;
    listen          [::]:80;
    server_name     post.example.com;
    return 404; # managed by Certbot
```


## Testing

You can send a test request to static_contact via curl:
```Bash
curl --header "Content-Type: application/json" --request POST --data '{"name":"Mr. Foo Bar", "email":"mrfoo@bar.com", "phone":"+49012345678", "message":"Media is the massage", "identifier":"mysiteidentifier"}' http://localhost:8088
```

This emulates a user with the name "Mr. Foo Bar", the email "mrfoo@bar.com" and the phone number "+49012345678" writing a message with the content "Media is the massage". Note the field `"identifier":"mysitename"` – this identifies the site against the server. If there is no endpoint with the identifier "mysiteidentifier" in the `config.toml`, the request is ignored. This message is checked for type and length, and will be sent via SMTP to the according `endpoint.target` email adress specified in the config.