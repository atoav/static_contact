# static_contact

static_contact is a server side tool that relays http POST requests with form fields like name, email, phone and message to email targets via SMTP.  
This is meant to be a simple self-hosted solution for contact forms on static websites. You add a form and a little bit of javascript to a static website, it POSTs the message to a different server, does a few checks and sends it to an target adress via email. It is possible to connect multiple static websites to one single instance of the service.

## Installation
static_contact runs as a systemd-service on a local port. Although it would be possible to expose this directly to the web, it is recommended to run this service behind a a reverse proxy server (like nginx).

Start the service via `sudo systemctl start static_contact`

## Operation

You can send a test request to static_contact via curl:
```Bash
curl --header "Content-Type: application/json" --request POST --data '{"name":"Mr. Foo Bar", "email":"mrfoo@bar.com", "phone":"+49012345678", "message":"Media is the massage", "identifier":"mysiteidentifier"}' http://localhost:8088
```

This emulates a user with the name "Mr. Foo Bar", the email "mrfoo@bar.com" and the phone number "+49012345678" writing a message with the content "Media is the massage". Note the field `"identifier":"mysitename"` – this identifies the site against the server. If there is no endpoint with the identifier "mysiteidentifier" in the `config.toml`, the request is ignored. This message is checked for type and length, and will be sent via SMTP to the according `endpoint.target` email adress specified in the config.