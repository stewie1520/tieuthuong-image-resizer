# Image Resizer Service

A high-performance Rust service for resizing images stored in AWS S3. The service downloads images from S3, resizes them according to specified dimensions and object modes, then uploads the resized version back to S3.

## Features

- **Multiple Object Modes**: Support for `cover`, `contain`, `fill`, and `scale-down` resize modes
- **S3 Integration**: Direct download and upload to AWS S3 buckets
- **Smart Caching**: Automatically checks if resized image exists and returns cached URL
- **Flexible S3 URL Support**: Accepts s3://, virtual-hosted style, and path-style URLs
- **High Performance**: Built with Rust and Tokio for async operations
- **RESTful API**: Simple HTTP endpoint for resize requests
- **Automatic Naming**: Resized images are automatically named with dimensions

## Object Modes

- **cover**: Scales the image to fill the target dimensions while maintaining aspect ratio. Crops excess parts.
- **contain**: Scales the image to fit within the target dimensions while maintaining aspect ratio. No cropping.
- **fill**: Stretches the image to exactly match the target dimensions. May distort aspect ratio.
- **scale-down**: Only scales down if the image is larger than target dimensions. Never scales up.

## Prerequisites

- Rust 1.70 or higher
- AWS credentials configured (via environment variables or AWS credentials file)
- Access to an S3 bucket

## Setup

1. **Clone the repository**
   ```bash
   cd /Users/hieudong/projects/tieuthuong2/image-resizer
   ```

2. **Configure AWS credentials**
   
   Set environment variables:
   ```bash
   export TT_AWS_ACCESS_KEY_ID=your_access_key
   export TT_AWS_SECRET_ACCESS_KEY=your_secret_key
   export TT_AWS_REGION=us-east-1
   ```
   
   **Note**: The service uses `TT_` prefixed environment variables instead of standard AWS variable names to support CI environments that restrict variables starting with `AWS_`.

3. **Build the project**
   ```bash
   cargo build --release
   ```

4. **Run the service**
   ```bash
   cargo run --release
   ```
   
   The service will start on `http://0.0.0.0:3000`

## API Usage

### Resize Image Endpoint

**POST** `/resize`

**Request Body:**
```json
{
  "s3_url": "s3://my-bucket/path/to/image.jpg",
  "width": 800,
  "height": 600,
  "object_mode": "cover"
}
```

**Parameters:**
- `s3_url` (required): S3 URL of the source image. Supports multiple formats:
  - `s3://bucket-name/key`
  - `https://bucket.s3.region.amazonaws.com/key`
  - `https://bucket.s3-region.amazonaws.com/key`
  - `https://s3.region.amazonaws.com/bucket/key`
- `width` (required): Target width in pixels (must be > 0)
- `height` (required): Target height in pixels (must be > 0)
- `object_mode` (optional): Resize mode - `cover`, `contain`, `fill`, or `scale-down` (default: `cover`)

**Caching Behavior:**
The service automatically checks if a resized image with the same dimensions already exists in S3. If found, it immediately returns the cached URL without reprocessing the image. This significantly improves performance and reduces costs for repeated requests.

**Response:**
```json
{
  "original_url": "s3://my-bucket/path/to/image.jpg",
  "resized_url": "s3://my-bucket/path/to/image_800x600.jpg",
  "width": 800,
  "height": 600,
  "object_mode": "cover"
}
```

### Example cURL Request

```bash
curl -X POST http://localhost:3000/resize \
  -H "Content-Type: application/json" \
  -d '{
    "s3_url": "s3://my-bucket/photos/vacation.jpg",
    "width": 1920,
    "height": 1080,
    "object_mode": "cover"
  }'
```

## Error Handling

The service returns appropriate HTTP status codes:

- `200 OK`: Successful resize operation
- `400 Bad Request`: Invalid S3 URL or parameters
- `422 Unprocessable Entity`: Image processing error
- `502 Bad Gateway`: S3 operation failed
- `500 Internal Server Error`: Unexpected server error

Error responses include a JSON body with details:
```json
{
  "error": "Error message description"
}
```

## Development

### Run in development mode
```bash
cargo run
```

### Run with debug logging
```bash
RUST_LOG=debug cargo run
```

### Run tests
```bash
cargo test
```

## Project Structure

```
image-resizer/
├── src/
│   ├── main.rs              # Application entry point
│   ├── handlers.rs          # HTTP request handlers
│   ├── models.rs            # Request/response models
│   ├── s3.rs                # S3 client and utilities
│   ├── image_processor.rs   # Image resizing logic
│   └── error.rs             # Error types and handling
├── Cargo.toml               # Dependencies and metadata
└── README.md                # This file
```

## Dependencies

- **axum**: Web framework
- **tokio**: Async runtime
- **aws-sdk-s3**: AWS S3 SDK
- **image**: Image processing library
- **serde**: Serialization/deserialization
- **tracing**: Logging and diagnostics

## Performance Considerations

- Images are processed in memory
- Large images may require significant memory
- Consider implementing size limits for production use
- The service uses Lanczos3 filtering for high-quality resizing

## Security Notes

- Ensure AWS credentials have appropriate S3 permissions (GetObject, PutObject)
- Consider implementing authentication for the API endpoint
- Validate S3 URLs to prevent unauthorized bucket access
- Consider rate limiting for production deployments

## License

MIT
