# FileSyncHub Documentation

This directory contains the documentation for FileSyncHub, hosted on GitHub Pages.

## Directory Structure

```
docs/
├── _config.yml          # Jekyll configuration
├── index.md            # Main landing page
├── api/                # API documentation
├── user-guide/         # User guides and tutorials
│   ├── index.md
│   ├── google-drive-setup.md
│   ├── onedrive-setup.md
│   └── sync-configuration.md
├── examples/           # Code examples and use cases
├── contributing/       # Contributing guidelines
└── assets/            # Images and other assets
```

## Local Development

1. Install Ruby and Bundler
   ```bash
   # For Ubuntu/Debian
   sudo apt-get install ruby ruby-dev bundler

   # For macOS
   brew install ruby
   gem install bundler
   ```

2. Install Jekyll and dependencies
   ```bash
   bundle install
   ```

3. Run the local server
   ```bash
   bundle exec jekyll serve
   ```

4. View the site at `http://localhost:4000/FileSyncHub/`

## Writing Documentation

### Adding New Pages

1. Create a new Markdown file in the appropriate directory
2. Add front matter at the top:
   ```yaml
   ---
   layout: default
   title: Your Page Title
   nav_order: 1
   ---
   ```

### Style Guide

- Use Markdown for formatting
- Include code examples in fenced code blocks
- Use relative links for internal navigation
- Add alt text to images
- Keep line length under 80 characters
- Use headers hierarchically (H1 > H2 > H3)

### Code Examples

Use syntax highlighting:

```rust
use filesynchub::GoogleDriveClient;

async fn example() -> Result<()> {
    let client = GoogleDriveClient::new("root").await?;
    Ok(())
}
```

### Images

Store images in `assets/` and reference them like this:

```markdown
![Alt text](/assets/images/example.png)
```

## Building Documentation

The documentation is automatically built and deployed when changes are pushed to the main branch.

### Manual Build

```bash
bundle exec jekyll build
```

The built site will be in `_site/`.

## Contributing to Documentation

1. Fork the repository
2. Create a new branch
3. Make your changes
4. Test locally
5. Submit a pull request

## Documentation TODOs

- [ ] Add API reference documentation
- [ ] Create more code examples
- [ ] Add troubleshooting guide
- [ ] Improve configuration documentation
- [ ] Add video tutorials

## Additional Resources

- [Jekyll Documentation](https://jekyllrb.com/docs/)
- [GitHub Pages Documentation](https://docs.github.com/en/pages)
- [Markdown Guide](https://www.markdownguide.org/) 