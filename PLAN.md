# Implement plan: Utilize bittorrent protocol using magnet links to distribute files efficiently across my managed local network.

*Scenario: Use for centralized exam. The operator needs to distribute exam questions in pdf, and some heavy resource files to all computers in local, isolated network*
*File size is approx. 2GB, need to be distributed completely in just 10 minutes. Every client is connected to an access switch with 1Gbps link. Server link speed is 10Gbps*

1. Spectifications:
   - Server acts as bootstrap node for the bittorrent network, providing magnet links to clients.
   - Clients can download files using the provided magnet links, sharing pieces of the files with each other.
   - Each share is a single file, not sharing a directory.
   - Utilize the throughput of the local network to get the files distributed as fast as possible.
   - Package project as a single executable for easy deployment on both server and clients.
2. Custom control plane:
   - Using HTTP API provided by the server to manage magnet links and track client activity.
   - Clients run as a daemon (foreground console window), fetch every 30 seconds to get the latest magnet links and report their download status.
   - Implement API for admins to add new files (no removal) and watch the download progress of each file.
   - The control plane will be hardcoded to `https://operator.local:12405`.
   - Logging: Basic console prints for the client, tracing on the server side.
3. Security and access control:
   - Use a self-signed certificate for secure communication between clients and server on the control plane.
   - `certificate.crt` will be bundled and kept in the root folder alongside the binary.
   - No need to implement per-client authentication.
