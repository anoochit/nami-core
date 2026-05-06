import argparse
import os
import sys
import base64
from pathlib import Path
import PIL.Image
from dotenv import load_dotenv
from google import genai
from google.genai import types

# Load environment variables from .env file if it exists
# load_dotenv()

# Setup the GenAI client
api_key = os.environ.get("GOOGLE_API_KEY")

# Initialize with API Key
if not api_key:
    print("Error: GOOGLE_API_KEY environment variable not set.")
    sys.exit(1)
client = genai.Client(api_key=api_key)
print("Using Gemini API Key.")

def resolve_cover_path(cover_arg: str) -> Path:
    """Resolves the cover path with project-relative priority."""
    p = Path(cover_arg)

    # 1) If user passed an absolute path or an existing relative path, use it
    if p.is_file():
        return p

    raise FileNotFoundError(
        f"Could not find cover image '{cover_arg}'. "
    )

def generate_mockup(cover_path, description, output_path):
    """Generates a photo-realistic book mockup from cover image and description."""
    # gemini-2.5-flash-image supports image input + image output
    # Note: This model usually requires Vertex AI.
    model_name = "gemini-3.1-flash-image-preview"

    try:
        # Load the cover image
        cover_img = PIL.Image.open(cover_path)

        # Construct the prompt
        prompt = (
            f"Generate a photo-realistic 3D book mockup. The book cover is the provided image. "
            f"Scene description: {description}. "
            "Ensure adjusted, natural lighting and high-quality textures."
        )

        print(f"Generating mockup with model '{model_name}'...")

        # Response modalities for image output
        response = client.models.generate_content(
            model=model_name,
            contents=[prompt, cover_img],
            config=types.GenerateContentConfig(
                response_modalities=["IMAGE", "TEXT"],
                image_config=types.ImageConfig(
                    aspect_ratio="1:1",
                ),
            ),
        )

        # Parse the response parts for image data
        image_saved = False
        if not response.candidates:
            print("No candidates returned in response.")
            sys.exit(1)

        for part in response.candidates[0].content.parts:
            if part.inline_data is not None:
                image_bytes = part.inline_data.data
                with open(output_path, "wb") as f:
                    f.write(image_bytes)
                image_saved = True
                print(f"Success: Mockup saved to '{output_path}'.")
                break

        if not image_saved:
            # Fallback: print any text response for debugging
            print("No image returned in response.")
            for part in response.candidates[0].content.parts:
                if hasattr(part, "text") and part.text:
                    print("Model response:", part.text)

    except Exception as e:
        print(f"Error generating mockup: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate a book mockup image.")
    parser.add_argument("--cover", required=True, help="Filename or path of the book cover image.")
    parser.add_argument("--description", required=True, help="Description for the mockup scene.")
    parser.add_argument("--output", help="Filename or path to save the generated mockup.")

    args = parser.parse_args()

    try:
        # Resolve cover path
        cover_path = resolve_cover_path(args.cover)
        print(f"Using cover image: {cover_path.resolve()}")
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)

    # Resolve output path
    if args.output:
        output_path = Path(args.output)
    else:
        output_path = Path(f"mockup_{cover_path.name}")

    generate_mockup(str(cover_path), args.description, str(output_path))
