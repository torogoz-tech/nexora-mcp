# Nexora MCP

Version 0.4.0 stateless Streamable HTTP gateway for the Nexora MCP endpoint.

The gateway accepts authenticated POST requests at /mcp and forwards them to one operator-configured API /mcp URL. The API remains authoritative for API-key authentication, authorization, tool catalog, and tool execution; the upstream API endpoint must stay private and reachable only from trusted services. This version separates ingress transport and MCP scaling, but tool execution still runs in the API.

Set NEXORA_MCP_UPSTREAM to the private API endpoint, such as http://nexora-api:18080/mcp. The gateway binds to 0.0.0.0:18081 by default. It rejects browser Origin requests, forwards the authorization and MCP protocol headers, bounds request size and concurrent upstream calls, and does not log credentials or request bodies.
