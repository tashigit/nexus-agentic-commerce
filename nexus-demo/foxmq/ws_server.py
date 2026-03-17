import asyncio
import json
import logging
from typing import List, Callable, Dict, Any

import websockets
from websockets.server import WebSocketServerProtocol

from nexus_sdk.types import AgentInfo, EventAgentCreated, Command

logger = logging.getLogger("ws_server")

class WSServer:
    def __init__(self, host: str, port: int, initial_agents: List[AgentInfo]):
        self.host = host
        self.port = port
        self.initial_agents = initial_agents
        self.clients: set[WebSocketServerProtocol] = set()
        self.on_command: Callable[[str], None] = None

    async def start(self):
        logger.info(f"[ws] listening on ws://{self.host}:{self.port}")
        async with websockets.serve(self._handle_client, self.host, self.port):
            await asyncio.Future()  # run forever

    async def _handle_client(self, websocket: WebSocketServerProtocol):
        self.clients.add(websocket)
        logger.info(f"[ws] new connection from {websocket.remote_address}")

        try:
            # Replay initial agents
            for agent in self.initial_agents:
                event = EventAgentCreated(agent=agent).model_dump_json(by_alias=True)
                await websocket.send(event)

            # Listen for commands
            async for message in websocket:
                try:
                    data = json.loads(message)
                    cmd = Command(**data)
                    if self.on_command:
                        self.on_command(cmd.cmd)
                except Exception as e:
                    logger.error(f"[ws] Error parsing command: {e}")

        except websockets.exceptions.ConnectionClosed:
            pass
        finally:
            self.clients.remove(websocket)
            logger.info(f"[ws] {websocket.remote_address} disconnected")

    async def broadcast_event(self, event_json: str):
        if not self.clients:
            return
            
        # Broadcast to all connected clients
        websockets.broadcast(self.clients, event_json)
