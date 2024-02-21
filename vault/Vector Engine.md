**Vector Engine** is a program with the sole purpose of creating videos with programmatic animations and effects.

## Technologies
In order to accomplish the monolithic task, **Vector Engine** will utilize a few awesome technologies:
### Vulkan
Vulkan will be used to create the optimized [[Renderer]]. It will be used to render out the final videos as well as render previews that will display within the editor.
### Tauri
Tauri will provide the web technology to build the [[Editor]]. This will use display previews rendered with Vulkan as well as send commands to the backend to trigger functionality such as a renders, build timelines, and configure elements.
### Deno Core
Deno core will allow us the build a TypeScript [[Runtime]] which will act as the scripting for programmatic animations and effects.
## Architecture
The [[Editor]] will operate as the main host for the application. Within the editor, a [[Project]] will be instantiated. This [[Project]] will hold both the [[Renderer]] and [[Runtime]] as well as a [[Timeline]] with [[Clips]]. The [[Editor]] will be able to send commands such as `render`, `create_clip`, `get_clips`, etc. to both display and modify the project. The [[Editor]] will have a timeline preview which will be rendered with the [[Renderer]] and sent over Tauri to the [[Editor]] as the [[Timeline]] is updated. See more in [[Architecture.canvas|Architecture]]