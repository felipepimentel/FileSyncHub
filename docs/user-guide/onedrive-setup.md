# OneDrive Setup Guide

This guide will walk you through the process of setting up OneDrive authentication for FileSyncHub.

## Prerequisites

- A Microsoft Account
- Access to [Azure Portal](https://portal.azure.com)
- FileSyncHub installed on your system

## Step 1: Register Your Application

1. Go to the [Azure Portal](https://portal.azure.com)
2. Navigate to "Azure Active Directory"
3. Select "App registrations" in the left sidebar
4. Click "New registration"
5. Fill in the application details:
   - Name: "FileSyncHub"
   - Supported account types: "Personal Microsoft accounts only"
   - Redirect URI: Select "Public client/native" and enter `http://localhost:8080`
6. Click "Register"

## Step 2: Configure API Permissions

1. In your app registration, go to "API permissions"
2. Click "Add a permission"
3. Select "Microsoft Graph"
4. Choose "Delegated permissions"
5. Add the following permissions:
   - `Files.ReadWrite`
   - `offline_access`
6. Click "Add permissions"

## Step 3: Get Application Credentials

1. Note down your "Application (client) ID" from the overview page
2. Go to "Authentication" in the left sidebar
3. Under "Advanced settings":
   - Enable "Allow public client flows"
   - Set "Default client type" to "Yes"
4. Click "Save"

## Step 4: Configure FileSyncHub

1. Create a configuration file for OneDrive:
   ```bash
   mkdir -p ~/.config/filesynchub/credentials
   ```

2. Create `~/.config/filesynchub/credentials/onedrive.json` with your client ID:
   ```json
   {
     "client_id": "your-client-id-here",
     "redirect_uri": "http://localhost:8080"
   }
   ```

## Step 5: First-Time Authentication

1. Run FileSyncHub with the TUI interface:
   ```bash
   filesynchub sync --tui
   ```

2. The application will initiate the OneDrive authentication process
3. Follow the authentication prompts in your browser
4. Grant the requested permissions
5. Return to FileSyncHub

## Troubleshooting

### Common Issues

1. **Authentication Failed**
   - Error: "AADSTS error"
   - Solution: Verify your client ID and ensure all permissions are correctly configured

2. **Connection Timeout**
   - Error: "Connection timed out"
   - Solution: Check your internet connection and try again

3. **Invalid Redirect URI**
   - Error: "Invalid redirect URI"
   - Solution: Ensure the redirect URI in your configuration matches the one in Azure Portal

### Security Best Practices

- Keep your client ID secure
- Never commit credentials to version control
- Use environment variables when possible
- Regularly review application permissions

## Advanced Configuration

### Environment Variables

You can use environment variables instead of configuration files:

```bash
export FILESYNCHUB_ONEDRIVE_CLIENT_ID="your-client-id"
export FILESYNCHUB_ONEDRIVE_REDIRECT_URI="http://localhost:8080"
```

### Custom Endpoints

For special cases (e.g., OneDrive for Business), you can configure custom endpoints:

```json
{
  "client_id": "your-client-id",
  "redirect_uri": "http://localhost:8080",
  "authority_url": "https://login.microsoftonline.com/your-tenant-id"
}
```

## Next Steps

- [Configure sync settings](sync-configuration.md)
- [Set up automated syncing](automated-sync.md)
- [Explore advanced features](advanced-features.md)

## Additional Resources

- [Microsoft Graph Documentation](https://docs.microsoft.com/en-us/graph/)
- [Azure AD Authentication](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- [OneDrive API Reference](https://docs.microsoft.com/en-us/onedrive/developer/rest-api/) 