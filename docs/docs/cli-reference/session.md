# ciab session

Manage agent sessions.

## create

Create a new session in a sandbox.

```bash
ciab session create --sandbox-id <id>
```

## list

List sessions for a sandbox.

```bash
ciab session list --sandbox-id <id>
```

## get

Get session details with message history.

```bash
ciab session get <session-id>
```

## send

Send a message to a session.

```bash
ciab session send <session-id> --message "Your message here"
```

## interrupt

Interrupt the agent in a session.

```bash
ciab session interrupt <session-id>
```
