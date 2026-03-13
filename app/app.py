from fastapi import FastAPI

from app.lifespan import lifespan


def create_app() -> FastAPI:
    app = FastAPI(lifespan=lifespan)
    return app


app = create_app()
