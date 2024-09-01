from .Client import Client

class FabricClient(Client):
    def __init__(self, name: str, link: str, version: str = '1.12.2', working: bool = True, id: int = 1) -> None:
        super().__init__(name, link, version, working, id)