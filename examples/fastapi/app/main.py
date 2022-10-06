from fastapi import FastAPI

app = FastAPI()


@app.get("/")
async def root():
    print("in root method")
    return {"message": "Hello World"}
