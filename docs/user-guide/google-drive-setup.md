# Google Drive Setup Guide

This guide will walk you through the process of setting up Google Drive authentication for FileSyncHub.

## Prerequisites

- A Google Account
- Access to [Google Cloud Console](https://console.cloud.google.com)
- FileSyncHub installed on your system

## Step 1: Create a Google Cloud Project

1. Go to the [Google Cloud Console](https://console.cloud.google.com)
2. Click on "Select a Project" at the top of the page
3. Click "New Project"
4. Enter a project name (e.g., "FileSyncHub")
5. Click "Create"

## Step 2: Enable the Google Drive API

1. Select your project in the Google Cloud Console
2. Go to the [API Library](https://console.cloud.google.com/apis/library)
3. Search for "Google Drive API"
4. Click on "Google Drive API"
5. Click "Enable"

## Step 3: Configure OAuth Consent Screen

1. Go to the [OAuth consent screen](https://console.cloud.google.com/apis/credentials/consent)
2. Select "External" user type
3. Fill in the required information:
   - App name: "FileSyncHub"
   - User support email: Your email
   - Developer contact information: Your email
4. Click "Save and Continue"
5. Add the following scope:
   - `https://www.googleapis.com/auth/drive.file`
6. Click "Save and Continue"
7. Add your email address as a test user
8. Click "Save and Continue"

## Step 4: Create OAuth 2.0 Credentials

1. Go to the [Credentials](https://console.cloud.google.com/apis/credentials) page
2. Click "Create Credentials"
3. Select "OAuth client ID"
4. Choose "Desktop app" as the application type
5. Name it "FileSyncHub Desktop Client"
6. Click "Create"
7. Download the client configuration file (JSON)

## Step 5: Configure FileSyncHub

1. Create a `credentials` directory in your FileSyncHub configuration folder:
   ```bash
   mkdir -p ~/.config/filesynchub/credentials
   ```

2. Copy the downloaded JSON file to the credentials directory:
   ```bash
   cp ~/Downloads/client_secret_*.json ~/.config/filesynchub/credentials/google_drive.json
   ```

## Step 6: First-Time Authentication

1. Run FileSyncHub with the TUI interface:
   ```bash
   filesynchub sync --tui
   ```

2. The application will show a popup with authentication instructions
3. A browser window should open automatically
4. If not, copy the provided URL and paste it into your browser
5. Sign in with your Google account
6. Grant the requested permissions
7. Return to FileSyncHub

## Troubleshooting

### Common Issues

1. **Authentication Timeout**
   - Error: "Authentication timed out after 60 seconds"
   - Solution: Restart FileSyncHub and try again, being sure to complete the process within 60 seconds

2. **Invalid Client Configuration**
   - Error: "Error reading client secret file"
   - Solution: Verify that the credentials file is correctly named and placed in the right directory

3. **Access Denied**
   - Error: "Access denied" during Google sign-in
   - Solution: Make sure your email is added as a test user in the OAuth consent screen

### Security Notes

- Keep your `google_drive.json` file secure
- Never share your credentials
- Use environment variables for sensitive information in production

## Next Steps

- [Configure file synchronization](sync-configuration.md)
- [Explore advanced features](advanced-features.md)
- [Set up automated syncing](automated-sync.md)

## Additional Resources

- [Google Cloud Documentation](https://cloud.google.com/docs)
- [OAuth 2.0 for Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app)
- [Google Drive API Reference](https://developers.google.com/drive/api/v3/reference) 