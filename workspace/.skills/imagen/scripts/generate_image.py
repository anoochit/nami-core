#!/usr/bin/env python3
import os
import sys
import json
import hashlib
from google import genai
from google.genai import types

def generate_image(prompt, aspect_ratio="1:1"):
    api_key = os.environ.get("GOOGLE_API_KEY")
    if not api_key:
        return {"status": "error", "message": "GOOGLE_API_KEY not found"}

    try:
        client = genai.Client(api_key=api_key)
        # Correct class name: GenerateImagesConfig
        response = client.models.generate_images(
            model='imagen-4.0-generate-001',
            prompt=prompt,
            config=types.GenerateImagesConfig(
                number_of_images=1,
                aspect_ratio=aspect_ratio,
                output_mime_type='image/png'
            )
        )
        
        if response.generated_images:
            image_obj = response.generated_images[0]
            # Check for image_bytes or PIL image
            if hasattr(image_obj.image, 'image_bytes') and image_obj.image.image_bytes:
                image_bytes = image_obj.image.image_bytes
            else:
                # Some versions might use 'image' directly if it's bytes or something else
                # Let's try to get it
                return {"status": "error", "message": f"Generated image object structure unknown: {type(image_obj.image)}"}
            
            h = hashlib.md5(prompt.encode()).hexdigest()[:8]
            filename = f"generated_{h}.png"
            
            with open(filename, "wb") as f:
                f.write(image_bytes)
                
            return {
                "status": "success", 
                "image_path": filename, 
                "prompt": prompt
            }
        else:
            return {"status": "error", "message": f"No images generated. Response: {response}"}
            
    except Exception as e:
        import traceback
        return {"status": "error", "message": str(e), "traceback": traceback.format_exc()}

if __name__ == "__main__":
    try:
        # Check if input is from stdin or args
        if not sys.stdin.isatty():
            input_data = sys.stdin.read().strip()
            if input_data:
                try:
                    args = json.loads(input_data)
                except:
                    args = {"prompt": input_data}
            else:
                args = {}
        else:
            args = {}
            
        prompt = args.get("prompt")
        if not prompt and len(sys.argv) > 1:
            prompt = sys.argv[1]
            
        aspect_ratio = args.get("aspect_ratio", "1:1")
        
        if not prompt:
            print(json.dumps({"status": "error", "message": "No prompt provided"}))
            sys.exit(1)
            
        result = generate_image(prompt, aspect_ratio)
        print(json.dumps(result))
    except Exception as e:
        import traceback
        print(json.dumps({"status": "error", "message": str(e), "traceback": traceback.format_exc()}))
        sys.exit(1)
