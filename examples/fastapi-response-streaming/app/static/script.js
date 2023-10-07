async function tellStory() {
  const story = document.getElementById("topic").value;

  if (story.trim().length === 0) {
    return;
  }

  const storyOutput = document.getElementById("story-output");
  storyOutput.innerText = "Thinking...";

  try {
    // Use Fetch API to send a POST request for response streaming. See https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API 
    const response = await fetch("/api/story", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ "topic": story })
    });

    storyOutput.innerText = "";

    // Response Body is a ReadableStream. See https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream
    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    // Process the chunks from the stream
    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }
      const text = decoder.decode(value);
      storyOutput.innerText += text;
    }

  } catch (error) {
    storyOutput.innerText = `Sorry, an error happened. Please try again later. \n\n ${error}`;
  }

}

document.getElementById("tell-story").addEventListener("click", tellStory);
document.getElementById('topic').addEventListener('keydown', function (e) {
  if (e.code === 'Enter') {
    tellStory();
  }
});
