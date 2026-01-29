import httpx
import pytest

from tests.conftest import gen_background_server_ctxmanager

# Docker container port
DOCKER_PORT = 8000
# SAM local API port
SAM_PORT = 3000


class TestDockerContainer:
    """Functional tests for the Docker container running directly."""

    @pytest.fixture(scope="class")
    def docker_server(self):
        """Start the Docker container and yield when ready."""
        docker_run_cmd = [
            "docker",
            "run",
            "--rm",
            "-p",
            f"{DOCKER_PORT}:8000",
            "hello-world-lambda",
        ]
        server_ctx = gen_background_server_ctxmanager(
            cmd=docker_run_cmd,
            port=DOCKER_PORT,
            healthendpoint="/health",
            wait_seconds=5,
        )
        with server_ctx() as process:
            yield process

    def test_docker_root_endpoint(self, docker_server):
        """Test the root endpoint returns Hello World."""
        response = httpx.get(f"http://localhost:{DOCKER_PORT}/")
        assert response.status_code == 200
        data = response.json()
        assert data == {"message": "Hello World"}

    def test_docker_health_endpoint(self, docker_server):
        """Test the health endpoint returns healthy status."""
        response = httpx.get(f"http://localhost:{DOCKER_PORT}/health")
        assert response.status_code == 200
        data = response.json()
        assert data == {"status": "healthy"}


class TestSAMLocal:
    """Functional tests for SAM local (Lambda simulation)."""

    @pytest.fixture(scope="class")
    def sam_server(self):
        """Start SAM local API and yield when ready."""
        sam_cmd = [
            "sam",
            "local",
            "start-api",
            "--port",
            str(SAM_PORT),
            "--warm-containers",
            "EAGER",
        ]
        server_ctx = gen_background_server_ctxmanager(
            cmd=sam_cmd,
            port=SAM_PORT,
            healthendpoint="/health",
            wait_seconds=5,
        )
        with server_ctx() as process:
            yield process

    def test_sam_root_endpoint(self, sam_server):
        """Test the root endpoint via SAM local returns Hello World."""
        response = httpx.get(f"http://localhost:{SAM_PORT}/")
        assert response.status_code == 200
        data = response.json()
        assert data == {"message": "Hello World"}

    def test_sam_health_endpoint(self, sam_server):
        """Test the health endpoint via SAM local returns healthy status."""
        response = httpx.get(f"http://localhost:{SAM_PORT}/health")
        assert response.status_code == 200
        data = response.json()
        assert data == {"status": "healthy"}
