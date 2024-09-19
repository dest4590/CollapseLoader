import socket
import threading

from collapse.modules.utils.Module import Module
from collapse.modules.utils.Client import Client
from collapse.modules.utils.ClientManager import client_manager


class Background(Module):
    def __init__(self, host='', port=9090):
        super().__init__()
        self.host = host
        self.port = port
        self.server_socket = None

    def start_server(self):
        self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server_socket.bind((self.host, self.port))
        self.server_socket.listen(1)
        self.info(f"Server started on {self.host}:{self.port}")

        while True:
            client_socket, client_address = self.server_socket.accept()
            self.debug(f"Connection from {client_address}")
            thread = threading.Thread(target=self.handle_requests, args=(client_socket,))
            thread.daemon = False
            thread.start()

    def get_argument(self, data: str, index: int) -> str:
        try:
            return data.split(':')[index]
        except IndexError:
            self.error(f"Argument at index {index} not found in data: '{data}'")
            return ""
        except ValueError:
            self.error(f"Unable to split data: '{data}'")
            return ""

    def handle_requests(self, client_socket: socket.socket):
        try:
            while True:
                try:
                    data = client_socket.recv(1024).decode('utf-8')
                    if not data:
                        break
                    
                    if data.startswith('client-run'):
                        name = self.get_argument(data, 1)
                        if name:
                            client = client_manager.get_client_by_name(name)
                            client.run()

                    elif data == 'stop-server':
                        self.server_socket.close()
                        break
                    
                    self.debug(f"Received: {data.strip()}")
                except socket.error as e:
                    self.error(f"Socket error: {e}")
                    break
                except Exception as e:
                    self.error(f"Error handling client: {e}")
                    break
        finally:
            client_socket.close()
            self.debug("Client connection closed")

server = Background()