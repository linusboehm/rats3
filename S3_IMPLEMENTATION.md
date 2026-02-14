# S3 Backend Implementation

## Overview

The S3 backend for rats3 has been fully implemented with **read-only access** to S3 buckets. This document explains the implementation details, security guarantees, and usage instructions.

## Security: Read-Only Guarantee

### AWS SDK Operations Used

The implementation uses **only** the following read-only AWS S3 operations:

1. **`ListObjectsV2`** - Lists objects in a bucket with a given prefix
   - Used in: `list()` and `download_folder()`
   - Permission required: `s3:ListBucket`
   - **Cannot modify or delete objects**

2. **`GetObject`** - Retrieves object content
   - Used in: `get_preview()`, `download_file()`, and `download_folder()`
   - Permission required: `s3:GetObject`
   - **Cannot modify or delete objects**

3. **`HeadObject`** - Retrieves object metadata (size, content-type)
   - Used in: `get_preview()` to check file size before downloading
   - Permission required: `s3:GetObject` (or `s3:ListBucket`)
   - **Cannot modify or delete objects**

### What's NOT Used (Write/Delete Operations)

The following operations are **deliberately NOT implemented** to ensure read-only access:

- ❌ `PutObject` - Upload objects
- ❌ `DeleteObject` - Delete objects
- ❌ `DeleteObjects` - Batch delete
- ❌ `CopyObject` - Copy objects
- ❌ `CreateBucket` - Create buckets
- ❌ `DeleteBucket` - Delete buckets
- ❌ Any write operations

### Code Review for Security

All S3 operations in `src/backend/s3.rs` have been implemented with explicit read-only comments:

```rust
/// List S3 objects at the given prefix (READ-ONLY operation)
/// Uses ListObjectsV2 which is a read-only S3 operation
async fn list(&self, prefix: &str) -> Result<ListResult>

/// Get preview of an S3 object (READ-ONLY operation)
/// Uses GetObject which is a read-only S3 operation
async fn get_preview(&self, path: &str, max_size: usize) -> Result<PreviewContent>

/// Download a single file from S3 (READ-ONLY operation)
/// Uses GetObject which is a read-only S3 operation
async fn download_file(&self, path: &str, destination: &Path) -> Result<()>

/// Download a folder from S3 recursively (READ-ONLY operation)
/// Uses ListObjectsV2 and GetObject which are read-only S3 operations
async fn download_folder(&self, prefix: &str, destination: &Path) -> Result<usize>
```

## Implementation Details

### 1. Listing S3 Objects (`list`)

**Purpose**: Browse S3 bucket contents like a directory tree

**Implementation**:
- Uses `list_objects_v2()` with delimiter `/` to create directory-like structure
- Automatically paginates through all results
- Separates "directories" (common prefixes) from files
- Sorts results: directories first, then by name (consistent with local backend)
- Skips directory markers (keys ending with `/`)

**Features**:
- Handles large buckets with automatic pagination
- Deduplicates directory entries
- Extracts file size and modification time metadata

### 2. Preview S3 Objects (`get_preview`)

**Purpose**: Show file contents in the preview pane

**Implementation**:
- First checks object size with `head_object()` to avoid downloading large files
- Returns `PreviewContent::TooLarge` if file exceeds max size
- Downloads object content with `get_object()`
- Attempts UTF-8 text conversion
- Falls back to binary indicator with MIME type for non-text files

**Features**:
- Respects configurable size limits (default: 10 MB)
- Detects text vs binary content
- Provides MIME type information for binary files
- Graceful error handling with error messages

### 3. Download Single File (`download_file`)

**Purpose**: Save a single S3 object to local filesystem

**Implementation**:
- Downloads object with `get_object()`
- Streams content to local file using tokio async I/O
- Creates destination file with proper error handling

**Features**:
- Async I/O for efficient downloads
- Proper error context for debugging
- Works with any S3 object size (limited by disk space)

### 4. Download Folder (`download_folder`)

**Purpose**: Recursively download all objects with a given prefix

**Implementation**:
- Lists all objects with the prefix using `list_objects_v2()` pagination
- Creates local directory structure matching S3 hierarchy
- Downloads each object individually
- Returns count of downloaded files

**Features**:
- Preserves directory structure
- Handles nested folders
- Skips directory markers
- Creates parent directories as needed
- Returns file count for user feedback

## Usage

### Prerequisites

1. **Rust version**: 1.91+ (required for AWS SDK)
2. **AWS credentials**: Configure via environment or `~/.aws/credentials`
3. **IAM permissions**: At minimum, your IAM role/user needs:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": [
           "s3:ListBucket",
           "s3:GetObject"
         ],
         "Resource": [
           "arn:aws:s3:::your-bucket-name",
           "arn:aws:s3:::your-bucket-name/*"
         ]
       }
     ]
   }
   ```

### Building with S3 Support

```bash
# Build with S3 feature enabled
cargo build --release --features s3

# Or run directly
cargo run --features s3 -- s3://bucket-name/prefix
```

### Running

```bash
# Browse entire bucket
./target/release/rats3 s3://my-bucket

# Browse specific prefix
./target/release/rats3 s3://my-bucket/data/2024/

# The trailing slash is optional
./target/release/rats3 s3://my-bucket/data/2024
```

### AWS Credentials

rats3 uses the standard AWS SDK credential chain, which looks for credentials in this order:

1. Environment variables: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
2. Web identity token credentials
3. ECS container credentials
4. EC2 instance metadata credentials
5. AWS credentials file: `~/.aws/credentials`
6. AWS config file: `~/.aws/config`

Example credentials file (`~/.aws/credentials`):
```ini
[default]
aws_access_key_id = YOUR_ACCESS_KEY
aws_secret_access_key = YOUR_SECRET_KEY
aws_session_token = YOUR_SESSION_TOKEN  # Optional, for temporary credentials

[profile-name]
aws_access_key_id = ANOTHER_ACCESS_KEY
aws_secret_access_key = ANOTHER_SECRET_KEY
```

To use a specific profile:
```bash
export AWS_PROFILE=profile-name
./target/release/rats3 s3://bucket-name
```

To use a specific region:
```bash
export AWS_REGION=us-west-2
./target/release/rats3 s3://bucket-name
```

## Navigation

Once in S3 browse mode, you can:

- **Arrow keys / j,k**: Move up/down through objects
- **Enter / l**: Navigate into a "directory" (prefix)
- **Left / h**: Navigate to parent prefix
- **/**: Search/filter objects
- **Y**: Copy current S3 URI to clipboard (e.g., `s3://bucket/prefix/file.txt`)
- **R**: Browse navigation history
- **Ctrl-C / Ctrl-Q**: Quit

## Features

### Directory-like Navigation

S3 doesn't have real directories, but rats3 creates a directory-like experience using prefixes and delimiters:

- Objects with common prefixes are grouped as "folders"
- You can navigate up and down the prefix hierarchy
- The UI shows a familiar file browser interface

### File Preview

Select any object to see a preview in the right pane:
- Text files show syntax-highlighted content
- Binary files show metadata (size, MIME type)
- Large files show size warning without downloading

### Smart Performance

- **Pagination**: Automatically handles buckets with millions of objects
- **Preview caching**: Once previewed, files are cached for instant re-access
- **Size limits**: Large files aren't downloaded for preview
- **Async I/O**: Non-blocking operations keep UI responsive

## Architecture Integration

### Backend Trait Implementation

The S3 backend implements the same `Backend` trait as the local filesystem backend:

```rust
#[async_trait]
pub trait Backend: Send + Sync {
    async fn list(&self, prefix: &str) -> Result<ListResult>;
    async fn get_preview(&self, path: &str, max_size: usize) -> Result<PreviewContent>;
    async fn download_file(&self, path: &str, destination: &Path) -> Result<()>;
    async fn download_folder(&self, prefix: &str, destination: &Path) -> Result<usize>;
    fn get_display_path(&self, prefix: &str) -> String;
    fn get_parent(&self, prefix: &str) -> Option<String>;
}
```

This abstraction means:
- The UI code is completely backend-agnostic
- Switching between local and S3 is seamless
- All features (search, history, preview) work identically
- Easy to add more backends in the future (Azure Blob, GCS, etc.)

### Async/Sync Bridge

- Backend operations are async (using tokio)
- UI rendering is synchronous (using ratatui)
- Event loop awaits backend operations between renders
- Keeps UI responsive during S3 operations

## Error Handling

The implementation provides clear error messages:

- **Network errors**: "Failed to list S3 objects: connection timeout"
- **Permission errors**: "Failed to get S3 object: Access Denied"
- **Missing objects**: Preview shows "Failed to access object: NoSuchKey"
- **Invalid URIs**: "URI must start with s3://"

All errors use `anyhow::Context` to provide helpful error chains.

## Testing

### Local Testing Without S3

Test the UI and navigation with the local backend:

```bash
cargo run -- --local /path/to/test/directory
```

### S3 Testing

1. Create a test bucket with public objects (or configure IAM properly)
2. Add some test files with different prefixes
3. Run rats3:
   ```bash
   cargo run --features s3 -- s3://test-bucket
   ```

### Verify Read-Only Access

To verify the implementation truly cannot write:

1. Review `src/backend/s3.rs` - no `PutObject`, `DeleteObject`, etc.
2. Check AWS CloudTrail logs - only see `ListBucket` and `GetObject` events
3. Use restrictive IAM policy allowing only read operations
4. The application will work fine with read-only permissions

## Performance Considerations

### Pagination
- S3 list operations return max 1,000 objects per page
- Implementation automatically paginates through all pages
- Large buckets may take time to list completely

### Preview Size Limits
- Default: 10 MB maximum for preview
- Configurable in `~/.config/rats3/config.toml`
- Prevents downloading huge files just for preview

### Caching
- Preview content is cached in memory
- Navigating back to a file shows cached preview instantly
- Cache is cleared when navigating to a different prefix

## Known Limitations

1. **S3 versioning**: Only shows latest version of objects
2. **Multipart uploads**: N/A (read-only)
3. **Object metadata**: Only shows size, last-modified, and content-type
4. **Bucket policies**: Application requires read permissions
5. **Large buckets**: Initial listing of buckets with millions of objects may be slow

## Future Enhancements

Possible future improvements (all read-only):

- [ ] Show object storage class (STANDARD, GLACIER, etc.)
- [ ] Support for S3 Select (query objects without downloading)
- [ ] Parallel downloads for folder downloads
- [ ] Download progress indicators
- [ ] Support for S3-compatible storage (MinIO, DigitalOcean Spaces)
- [ ] Filter by object metadata (size, date range)
- [ ] Batch download with wildcard patterns

## Troubleshooting

### "Failed to list S3 objects: Access Denied"

**Solution**: Check your IAM permissions include `s3:ListBucket` for the bucket.

### "Failed to get S3 object: NoSuchKey"

**Solution**: The object may have been deleted. Refresh the listing.

### "URI must start with s3://"

**Solution**: Ensure you're using the correct URI format: `s3://bucket-name/prefix`

### AWS credentials not found

**Solution**: Configure AWS credentials via `~/.aws/credentials` or environment variables.

### Wrong region

**Solution**: Set `AWS_REGION` environment variable or configure in `~/.aws/config`.

## Security Best Practices

1. **Use IAM roles**: Instead of access keys, use IAM roles when running on EC2/ECS
2. **Principle of least privilege**: Grant only `s3:ListBucket` and `s3:GetObject` permissions
3. **Bucket policies**: Restrict access to specific prefixes if needed
4. **MFA**: Enable MFA for AWS console access
5. **Audit logs**: Monitor CloudTrail for unexpected access patterns
6. **Temporary credentials**: Use STS temporary credentials when possible

## Conclusion

The S3 backend implementation provides secure, read-only access to S3 buckets with a familiar file browser interface. All operations use only AWS read APIs, ensuring data safety while providing powerful navigation and preview capabilities.

**Key security guarantee**: It is impossible for this implementation to modify or delete S3 objects, as no write/delete operations are implemented in the code.
