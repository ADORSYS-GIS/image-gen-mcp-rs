# Image MCP Server

A Model Context Protocol (MCP) server for image generation and editing via OpenAI-compatible endpoints. Supports **nanobanana**, **DALL-E**, and **GPT Image** providers with flavor-specific features.

> **Deployment status: evaluated, not wired.** This server is not currently exposed
> through the ADORSYS-GIS AI platform gateway (`charts/mcps`). It was evaluated as a
> sixth MCP route and deliberately parked ([Story ai-helm#991](https://github.com/ADORSYS-GIS/ai-helm/issues/991)):
> image generation is already delivered by the self-hosted **Z-Image-Turbo** on the GPU
> fleet (LocalAI, ADR-0100/0102) via LibreChat's `IMAGE_GEN`, and the binary is a full
> major + 7 minors behind its core dependencies ([issue #41](https://github.com/ADORSYS-GIS/image-gen-mcp-rs/issues/41)),
> with default providers that are external SaaS (OpenAI / Gemini).
>
> To wire it later: finish the dependency migration (#41), rebuild and push the GHCR
> image, then add a `selfHosted` entry to `charts/mcps` pointing `OPENAI_BASE_URL` at
> the in-cluster z-turbo (or a federated image backend).

## Features

- 🎨 **Multi-flavor support** - Different parameter sets for Google Gemini and OpenAI
- 🔄 **Dual transport** - STDIO for local tools, HTTP/SSE for remote access
- ⚡ **High performance** - Built with Rust 2024 edition and mimalloc
- 🧩 **SOLID architecture** - Clean separation of concerns with MVP pattern
- 🔧 **Flexible configuration** - CLI flags or environment variables
- 💾 **Persistence** - Support for saving images locally in `file` mode

## Installation

### Docker (Recommended)

Pull the image from GitHub Container Registry:

```bash
docker pull ghcr.io/adorsys-gis/image-gen-mcp-rs:master
```

### Build from Source

```bash
# Build from source
cargo build --release

# Binary location
./target/release/image-mcp
```

## Quick Start with Docker

### STDIO Mode (for MCP Clients like Roo Code)

```bash
docker run --rm -i \
  --entrypoint /app/image-mcp \
  -e OPENAI_API_KEY=your-api-key \
  -e NANO_BANANA=true \
  ghcr.io/adorsys-gis/image-gen-mcp-rs:master \
  --transport-mode stdio
```

### HTTP/SSE Mode (for MCP Inspector)

```bash
# Start the HTTP server
docker run --rm -d -p 8080:8000 \
  -e OPENAI_API_KEY=your-api-key \
  -e NANO_BANANA=true \
  ghcr.io/adorsys-gis/image-gen-mcp-rs:master

# SSE endpoint: http://localhost:8080/sse
# HTTP endpoint: http://localhost:8080/mcp
```


### MCP Client Configuration

Configure your MCP client (Claude Desktop, Roo Code, Continue, etc.) to use the Docker image.

**Option 1: STDIO Mode (Recommended)**

Add to your MCP settings file:

```json
{
  "mcpServers": {
    "image-mcp": {
      "command": "docker",
      "args": [
        "run", "--rm", "-i",
        "--entrypoint", "/app/image-mcp",
        "-e", "OPENAI_API_KEY",
        "-e", "NANO_BANANA=true",
        "ghcr.io/adorsys-gis/image-gen-mcp-rs:master",
        "--transport-mode", "stdio"
      ],
      "env": {
        "OPENAI_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

**Option 2: SSE Mode (HTTP)**

For SSE mode, run the container separately first:

```bash
# Start the HTTP server
docker run --rm -d --name image-mcp-server -p 8080:8000 \
  -e OPENAI_API_KEY=your-api-key \
  -e NANO_BANANA=true \
  ghcr.io/adorsys-gis/image-gen-mcp-rs:master
```

Then configure your MCP client:

```json
{
  "mcpServers": {
    "image-mcp": {
      "type": "sse",
      "url": "http://localhost:8080/sse"
    }
  }
}
```

> **Note:** STDIO mode is recommended because it's simpler - the container automatically starts when your MCP client loads and stops when it closes. SSE mode requires manual server management but allows sharing the server across multiple clients.


## Quick Start (Binary)

### Basic Usage (STDIO)

```bash
# Set your API key and run
export OPENAI_API_KEY=your-api-key-here
./image-mcp

# Or pass directly
./image-mcp --api-key your-api-key-here
```

### HTTP Mode

```bash
# Run as HTTP server with SSE support
./image-mcp --api-key your-key --transport-mode http --port 8080

# Server available at http://127.0.0.1:8080
```

## Configuration

### CLI Flags

| Flag | ENV Variable | Default | Description |
|------|--------------|---------|-------------|
| `--api-key` | `OPENAI_API_KEY` | *required* | Your OpenAI-compatible API key |
| `--base-url` | `OPENAI_BASE_URL` | `https://api.openai.com/v1` | API endpoint URL |
| `--image-model` | `IMAGE_MODEL` | `nano-banana` | Default image model |
| `--transport-mode` | `TRANSPORT_MODE` | `stdio` | Transport: `stdio` or `http` |
| `--host` | `HOST` | `127.0.0.1` | HTTP server host |
| `--port` | `PORT` | `8080` | HTTP server port |
| `--nano-banana` | `NANO_BANANA` | `false` | Enable nano-banana flavor |
| `--openai-gen` | `OPENAI_GEN` | `false` | Enable OpenAI generation flavor |
| `--output-format` | `OUTPUT_FORMAT` | `url` | Output mode: `url` or `file` |
| `--output-dir` | `OUTPUT_DIR` | `./outputs` | Directory to save images in `file` mode |

### Example Configurations

```bash
# Custom API endpoint
./image-mcp --api-key sk-xxx --base-url https://your-server.com/v1

# Custom model
./image-mcp --api-key sk-xxx --image-model dall-e-3
```

## Flavors

Flavors activate provider-specific features. Only **one flavor** can be active at a time.

### Standard Flavor (Default)

Basic image generation with size control. Works with most providers.

```bash
./image-mcp --api-key sk-xxx
```

**Available Tools:**
- `generate_image` - Generate with prompt and size
- `continue_edit` - Iterative editing
- `list_models` - Show available models

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `prompt` | string | Text description of desired image |
| `model` | string? | Override default model |
| `size` | string? | Image size: `256`, `512`, `1024`, `1792x1024`, `1024x1792` |
| `n` | number? | Number of images (1-10) |

**Example:**
```json
{
  "prompt": "A serene mountain landscape at sunset",
  "size": "1024",
  "n": 1
}
```

### Nano-Banana Flavor (Google Gemini Native)

Native support for Google Gemini Image Generation. Optimized with **aspect ratio** and **seed** support.

```bash
# Enable via flag
./image-mcp --api-key sk-xxx --nano-banana

# Or via environment
NANO_BANANA=true ./image-mcp --api-key sk-xxx
```

**Available Tools:**
- `generate_image_nano` - Generate with ratio and seed
- `continue_edit` - Iterative editing (Session-based)
- `list_models` - Show available Gemini models

**Default Model Mappings:**
- `nano-banana` -> `gemini-2.5-flash-image`
- `nano-banana2` -> `gemini-3.1-flash-image-preview`
- `nano-banana-pro` -> `nano-banana-pro-preview`

**Parameters:**
| `prompt` | string | Text description of desired image |
| `model` | string? | Override default model |
| `ratio` | string? | Aspect ratio: `1:1`, `16:9`, `9:16`, `4:3`, `3:4`, `21:9` |
| `n` | number? | Number of images |
| `seed` | number? | Seed for reproducible results |

**JSON Response Format:**
All tools now return a structured JSON object for better AI integration:
```json
{
  "id": "cuid2_session_id",
  "url": "https://...",
  "prompt": "The original prompt used"
}
```

**Use Cases:**

1. **Consistent style across generations:**
   ```json
   {"prompt": "Product photo of sneakers", "ratio": "1:1", "seed": 12345}
   ```
   Same seed + similar prompt = consistent style

2. **Wide banners and headers:**
   ```json
   {"prompt": "Website hero image with gradient", "ratio": "21:9"}
   ```

3. **Social media content:**
   ```json
   {"prompt": "Instagram story background", "ratio": "9:16"}
   ```

### OpenAI-Gen Flavor

Full OpenAI GPT-Image support with **quality** and **style** parameters for generation, and **semantic image editing** for iterative refinement.

```bash
# Enable via flag
./image-mcp --api-key sk-xxx --openai-gen

# Or via environment
OPENAI_GEN=true ./image-mcp --api-key sk-xxx
```

**Available Tools:**
- `generate_image_openai` - Generate with quality and style
- `continue_edit` - **Semantic image editing** (understands prompts like "add a hat")
- `list_models` - Show available GPT-Image models

**Supported Models:**
| Model | Description |
|-------|-------------|
| `gpt-image-1` | Default - Best for generation and semantic editing |
| `gpt-image-1.5` | Latest model with improved capabilities |
| `gpt-image-1-mini` | Faster, cost-efficient variant |
| `dall-e-3` | Generation only (no edit support) |
| `dall-e-2` | Generation only (no semantic edit support) |

> **Note:** Only GPT-Image models support semantic image editing via `continue_edit`. DALL-E models have inpainting-only edit support which doesn't understand prompts like "add a rose to the chair".

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `prompt` | string | Text description of desired image |
| `model` | string? | Model: `gpt-image-1`, `gpt-image-1.5`, `gpt-image-1-mini`, `dall-e-3`, `dall-e-2` |
| `size` | string? | Size: `1024x1024`, `1792x1024`, `1024x1792` |
| `n` | number? | Number of images (DALL-E-3 only supports 1) |
| `quality` | string? | GPT-Image: `low`, `medium`, `high`, `auto`; DALL-E: `standard`, `hd` |
| `style` | string? | DALL-E-3 only: `vivid` or `natural` (not supported by GPT-Image) |

> **Note:** The `style` parameter is only supported by DALL-E-3. GPT-Image models do not accept this parameter.

**Example (GPT-Image):**
```json
{
  "prompt": "A photorealistic portrait of a wolf in forest",
  "model": "gpt-image-1",
  "size": "1024x1024",
  "quality": "high"
}
```

**Example (DALL-E-3):**
```json
{
  "prompt": "A photorealistic portrait of a wolf in forest",
  "model": "dall-e-3",
  "size": "1024x1024",
  "quality": "hd",
  "style": "natural"
}
```

**Use Cases:**

1. **High-quality marketing materials (GPT-Image):**
   ```json
   {"prompt": "Product shot of luxury watch", "quality": "high"}
   ```

2. **Creative and artistic images (DALL-E-3):**
   ```json
   {"prompt": "Abstract digital art with neon colors", "model": "dall-e-3", "quality": "hd", "style": "vivid"}
   ```

3. **Landscape orientations for presentations:**
   ```json
   {"prompt": "Tech conference background", "size": "1792x1024"}
   ```

## Case-by-Case Usage Examples

### Case 1: Content Creator - Blog Images

**Scenario:** You need consistent, reproducible blog header images.

**Recommended Setup:**
```bash
NANO_BANANA=true ./image-mcp --api-key sk-xxx --image-model nano-banana
```

**Workflow:**
1. Find a seed that produces your desired style
2. Use the same seed across all generations for consistency

```json
// First image
{"prompt": "Abstract geometric pattern for tech blog", "ratio": "16:9", "seed": 9000}

// Second image with same style
{"prompt": "Abstract pattern for AI article", "ratio": "16:9", "seed": 9000}

// Different ratio, same style seed
{"prompt": "Abstract pattern for sidebar", "ratio": "1:1", "seed": 9000}
```

### Case 2: Social Media Manager

**Scenario:** Create content for multiple platforms with different aspect ratios.

**Recommended Setup:**
```bash
NANO_BANANA=true ./image-mcp --api-key sk-xxx
```

**Workflow:**
```json
// Instagram post (square)
{"prompt": "Coffee shop morning vibes", "ratio": "1:1"}

// Instagram story (vertical)
{"prompt": "Coffee shop morning vibes", "ratio": "9:16"}

// Twitter/X header (wide)
{"prompt": "Coffee shop morning vibes", "ratio": "16:9"}

// YouTube thumbnail (standard)
{"prompt": "Coffee shop morning vibes", "ratio": "16:9"}
```

### Case 3: Product Photography

**Scenario:** High-quality product images for e-commerce.

**Recommended Setup:**
```bash
OPENAI_GEN=true ./image-mcp --api-key sk-xxx
```

**Workflow:**
```json
// Hero product shot
{
  "prompt": "Minimalist white background product photo of wireless headphones",
  "size": "1024x1024",
  "quality": "hd",
  "style": "natural"
}

// Lifestyle product shot
{
  "prompt": "Wireless headphones on wooden desk with coffee and laptop",
  "size": "1792x1024",
  "quality": "hd",
  "style": "natural"
}
```

### Case 4: UI/UX Designer

**Scenario:** Generate UI mockups and placeholder images.

**Recommended Setup:**
```bash
NANO_BANANA=true ./image-mcp --api-key sk-xxx
```

**Workflow:**
```json
// Mobile app mockup
{"prompt": "Modern fintech app dashboard with charts and cards", "ratio": "9:16"}

// Web dashboard placeholder
{"prompt": "Analytics dashboard with graphs and KPIs", "ratio": "16:9"}

// Icon exploration
{"prompt": "Minimalist app icon for fitness app", "ratio": "1:1", "n": 4}
```

### Case 5: Marketing Team

**Scenario:** Generate campaign visuals with consistent brand style.

**Recommended Setup:**
```bash
OPENAI_GEN=true ./image-mcp --api-key sk-xxx
```

**Workflow:**
```json
// Brand hero image
{
  "prompt": "Luxury brand aesthetic with gold accents and elegant typography space",
  "quality": "hd",
  "style": "vivid"
}

// Social campaign image
{
  "prompt": "Summer sale promotional image with bright colors",
  "quality": "hd",
  "style": "vivid"
}

// Email header
{
  "prompt": "Newsletter header with subtle brand elements",
  "quality": "standard",
  "style": "natural"
}
```

### Case 6: Game Developer

**Scenario:** Concept art and asset generation.

**Recommended Setup:**
```bash
NANO_BANANA=true ./image-mcp --api-key sk-xxx
```

**Workflow:**
```json
// Character concept
{"prompt": "Fantasy warrior character concept art detailed", "ratio": "9:16", "seed": 777}

// Environment concept
{"prompt": "Mystical forest environment concept art", "ratio": "16:9", "seed": 777}

// Item icons
{"prompt": "Magical sword icon pixel art style", "ratio": "1:1", "n": 4}
```

### Case 7: Rapid Prototyping

**Scenario:** Quick mockups and idea visualization.

**Recommended Setup:**
```bash
./image-mcp --api-key sk-xxx  # Standard flavor
```

**Workflow:**
```json
// Quick visualization
{"prompt": "App landing page with hero section and features grid", "size": "1792x1024"}

// Alternative variations
{"prompt": "App landing page alternative design", "size": "1792x1024", "n": 2}
```

### Case 8: Iterative Refinement (Session-Based)

**Scenario:** You generated an image and want to refine it (e.g., "add a hat").

**Workflow:**
1. Generate the initial image. The server returns an `image_id`.
2. Use the `continue_edit` tool with that `image_id` and your new prompt.

```json
// 1. Initial generation
tool: generate_image_nano {"prompt": "A cute cat on a sofa"}
response: {"id": "v7n9x...", "url": "..."}

// 2. Refinement using the ID
tool: continue_edit {
  "image_id": "v7n9x...",
  "prompt": "Now make the cat wear a blue wizard hat"
}
```
*Note: The server uses multimodal context (Gemini) or semantic image editing (GPT-Image models) to maintain consistency. DALL-E models do not support semantic editing.*

## Integration with MCP Clients

### Claude Desktop

Add to your Claude Desktop configuration:

**MacOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "image-mcp": {
      "command": "/path/to/image-mcp",
      "args": ["--api-key", "your-api-key", "--nano-banana"]
    }
  }
}
```


### HTTP Integration

For HTTP mode, configure your MCP client to connect to:

```
http://127.0.0.1:8080/mcp
```

With SSE support for real-time updates.



## Project Structure

```
src/
├── main.rs              # Entry point, dependency wiring
├── cli.rs               # CLI parsing with clap
├── core/                # Domain layer (Model)
│   ├── config.rs        # Configuration types
│   ├── errors.rs        # Domain errors
│   └── traits.rs        # Port interfaces
├── infrastructure/      # External adapters
│   ├── openai/          # OpenAI client implementation
│   └── id.rs            # CUID2 generator
├── application/         # Use cases
│   └── image.rs         # Image generation service
└── adapter/             # MCP protocol layer
    ├── handler.rs       # Server handler
    ├── handlers/        # Flavor-specific handlers
    └── tools.rs         # MCP tool definitions
```

## Architecture

- **SOLID Principles:** Single responsibility, open/closed, Liskov substitution, interface segregation, dependency inversion
- **MVP Pattern:** Model (core), View (MCP responses), Presenter (adapter/handler)
- **Clean Architecture:** Dependencies point inward (infrastructure → core)

## Development

```bash
# Check compilation
cargo check

# Build release
cargo build --release

# Run tests
cargo test
```

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Ensure all files stay under 150 lines
4. Submit a pull request
