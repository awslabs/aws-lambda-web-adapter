from threading import Thread
from queue import Queue
import json
import os
import requests


class BackgroundTaskExtension(Thread):
    def __init__(self):
        super().__init__()
        self.daemon = True
        self.queue = Queue()
        self.session = requests.Session()
        self.start()

    def run(self):
        # start an internal extension
        response = self.session.post(
            url=f"http://{os.environ['AWS_LAMBDA_RUNTIME_API']}/2020-01-01/extension/register",
            json={'events': ['INVOKE'],},
            headers={'Lambda-Extension-Name': 'background-task-extension' }
        )
        extension_id = response.headers['Lambda-Extension-Identifier']
        while True:
            response = self.session.get(
                url=f"http://{os.environ['AWS_LAMBDA_RUNTIME_API']}/2020-01-01/extension/event/next",
                headers={'Lambda-Extension-Identifier': extension_id},
                timeout=None
            )
            event = json.loads(response.text)
            if event['eventType'] == 'INVOKE':
                while True:
                    message = self.queue.get()
                    if message['type'] == 'TASK':
                        task, args, kwargs = message['task']
                        task(*args, **kwargs)
                    if message['type'] == 'DONE':
                        break

    def add_task(self, background_task, *args, **kwargs):
        self.queue.put( {"type": "TASK",
                         "task": (background_task, args, kwargs)} )

    def done(self):
        self.queue.put( {"type": "DONE"} )
