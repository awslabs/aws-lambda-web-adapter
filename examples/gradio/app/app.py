import gradio as gr

# Gradio Function
def greet(name, intensity):
    return "Hello, " + name + "!" * int(intensity)



# Define a no-op flagging callback
class NoOpFlaggingCallback(gr.FlaggingCallback):
    def setup(self, components, flagging_dir):
        pass  # Override setup to prevent directory creation

    def flag(self, flag_data, flag_option=None, flag_index=None, username=None):
        pass  # Do nothing when a flag is submitted


# Interface
demo = gr.Interface(
    fn=greet,
    inputs=["text", "slider"],
    outputs=["text"],
    title="Hello world",
    description="helper for aws-lambda-adapters",
    flagging_options=[],
    flagging_callback=NoOpFlaggingCallback(),
)

demo.launch(
        server_name="0.0.0.0",
        server_port=8080,
        enable_monitoring=None,
        share=False,
        debug=False
)