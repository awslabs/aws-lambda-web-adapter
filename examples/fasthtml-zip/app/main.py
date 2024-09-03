import secrets
from random import randint
from fasthtml.common import *

# Create a FastHTML application, setting the secret_key to avoid creating the .sesskey file on read only Lambda fs
app, rt = fast_app(
    secret_key=secrets.token_hex(32)
)

@rt
def index():
    """
    Route for the homepage.
    Displays a title and a button that generates a random integer when clicked.
    """
    button = Button('Click me for a random number', post=generate_random_number, target_id='random-number')
    number = Div(P('Placeholder'), id='random-number')
    card = Card(header=button, footer=number)
    return Titled('Hello World',  card)# Title of the page

@rt
def generate_random_number():
    """
    Route to generate and display a random integer.
    Replaces the content of the placeholder with the generated number.
    """
    random_integer = randint(1, 100)  # Generate a random integer between 1 and 100
    return P(f'Random number: {random_integer}')

# Serve the application on port 8000
serve(port=8000, reload=True)
