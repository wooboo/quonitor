# Quonitor - Multi-Provider LLM Quota Monitor

A cross-platform desktop application for monitoring LLM API quota usage across multiple providers (OpenAI, Anthropic, Google, GitHub Copilot).

## Features

- **Real-time Quota Monitoring**: Automatically fetches quota data every 5 minutes (configurable)
- **Multi-Provider Support**:
  - OpenAI (with full per-model tracking)
  - Anthropic/Claude (with per-model tracking)
  - Google/Vertex AI (basic support)
  - GitHub Copilot (basic support)
- **Per-Model Breakdown**: See usage and costs broken down by individual models (where provider supports it)
- **System Tray Integration**: Minimize to tray with quick access
- **Desktop Notifications**: Configurable alerts at 75%, 90%, and 95% usage thresholds
- **Historical Trends**: View usage charts over time (7, 30, or 90 days)
- **Secure Credential Storage**: API keys encrypted and stored in system keychain
- **Cost Tracking**: Automatic cost calculations based on provider pricing
- **Dark Mode UI**: Modern, clean interface with Tailwind CSS

## Prerequisites

- **Node.js** (v18 or higher)
- **Rust** (latest stable)
- **System Dependencies**:
  - Linux: `webkit2gtk`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
    ```bash
    # Debian/Ubuntu
    sudo apt install libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

    # Arch Linux
    sudo pacman -S webkit2gtk openssl libayatana-appindicator librsvg
    ```
  - Windows: No additional dependencies
  - macOS: No additional dependencies

## Installation

### Development Setup

1. **Clone the repository**
   ```bash
   cd /path/to/quonitor
   ```

2. **Install frontend dependencies**
   ```bash
   npm install
   ```

3. **Create placeholder icons** (temporary - replace with actual icons later)
   ```bash
   # Create basic PNG files for icons
   # You'll need to create these files or use a tool to generate them:
   # - src-tauri/icons/32x32.png
   # - src-tauri/icons/128x128.png
   # - src-tauri/icons/128x128@2x.png
   # - src-tauri/icons/icon.icns (macOS)
   # - src-tauri/icons/icon.ico (Windows)
   # - src-tauri/icons/tray-icon.png
   ```

   For now, you can use any PNG image and copy it to these locations, or use ImageMagick:
   ```bash
   # Create a simple colored square as placeholder
   convert -size 32x32 xc:#3b82f6 src-tauri/icons/32x32.png
   convert -size 128x128 xc:#3b82f6 src-tauri/icons/128x128.png
   convert -size 256x256 xc:#3b82f6 src-tauri/icons/128x128@2x.png
   convert -size 256x256 xc:#3b82f6 src-tauri/icons/tray-icon.png
   ```

4. **Run in development mode**
   ```bash
   npm run tauri dev
   ```

### Production Build

```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Usage

### Adding an Account

1. Click the **"Add Account"** button in the top right
2. Select your provider (OpenAI, Anthropic, etc.)
3. Enter a friendly name (e.g., "Work Account", "Personal")
4. Paste your API key
5. Click **"Add Account"**

The app will immediately fetch quota data for the new account.

### Viewing Quota Data

Each account is displayed as a card showing:
- Total input/output tokens used
- Estimated cost (last 24 hours)
- Per-model breakdown (expandable, for providers that support it)
- Historical trend charts (expandable)

### Configuring Settings

Click the **Settings** icon to configure:
- **Refresh Interval**: How often to check quota (default: 300 seconds)
- **Notifications**: Enable/disable desktop notifications
- **Notification Thresholds**: Choose which usage levels trigger alerts (75%, 90%, 95%)
- **Data Retention**: How long to keep historical data (default: 90 days)

### System Tray

The app can minimize to the system tray. Right-click the tray icon for options:
- **Show Window**: Bring the main window to front
- **Refresh Now**: Force an immediate quota refresh
- **Quit**: Exit the application

## API Provider Setup

### OpenAI

1. Get your API key from https://platform.openai.com/api-keys
2. The app uses the `/v1/organization/usage/completions` endpoint
3. Per-model tracking is fully supported

### Anthropic/Claude

1. Get your API key from https://console.anthropic.com/settings/keys
2. The app uses the `/v1/organization/usage` endpoint
3. Per-model tracking is supported (tested with current API)

### Google/Vertex AI

1. Requires OAuth 2.0 authentication (OAuth flow not fully implemented yet)
2. Basic support is in place, full implementation coming soon

### GitHub Copilot

1. Requires GitHub OAuth + PAT (OAuth flow not fully implemented yet)
2. Basic support is in place, full implementation coming soon

## Architecture

### Backend (Rust)

- **Database**: SQLite with SQLx for persistent storage
- **Providers**: Modular provider system with a common `QuotaProvider` trait
- **Services**:
  - **Scheduler**: Manages periodic quota fetching (tokio-based)
  - **Aggregator**: Fetches and stores data from all providers
  - **Notifier**: Desktop notifications with threshold tracking
  - **Cache**: In-memory cache for quick UI updates
- **Security**: API keys encrypted with AES-256-GCM, stored in OS keychain

### Frontend (React + TypeScript)

- **UI Framework**: React with Tailwind CSS
- **State Management**: TanStack Query for server state
- **Charts**: Recharts for historical trends
- **Icons**: Lucide React

## Data Storage

- **Database**: `~/.local/share/quonitor/quonitor.db` (Linux)
- **Logs**: `~/.local/share/quonitor/logs/` (future feature)
- **API Keys**: System keychain (via `keyring` crate)

## Troubleshooting

### "Failed to connect to database"

Make sure the app data directory exists and is writable:
```bash
mkdir -p ~/.local/share/quonitor
```

### "Failed to access keyring"

On Linux, ensure you have a keyring service running (e.g., `gnome-keyring`, `kwallet`).

### Icons not showing

Make sure all required icon files exist in `src-tauri/icons/`. See installation instructions above.

### Notifications not working

- **Linux**: Ensure you have a notification daemon running (most DEs include one)
- **Windows**: Notifications should work out of the box on Windows 10/11
- Check that notifications are enabled in Settings

## Development

### Project Structure

```
quonitor/
├── src/                      # Frontend (React)
│   ├── components/          # UI components
│   ├── hooks/               # React hooks
│   ├── styles/              # CSS files
│   └── types.ts             # TypeScript types
├── src-tauri/               # Backend (Rust)
│   └── src/
│       ├── api/             # Tauri commands
│       ├── db/              # Database layer
│       ├── providers/       # Provider adapters
│       ├── services/        # Background services
│       ├── crypto.rs        # Encryption
│       ├── error.rs         # Error handling
│       ├── tray.rs          # System tray
│       └── main.rs          # Entry point
└── README.md
```

### Adding a New Provider

1. Create `src-tauri/src/providers/yourprovider.rs`
2. Implement the `QuotaProvider` trait
3. Add pricing calculation logic
4. Register in `src-tauri/src/providers/mod.rs`
5. Test with real API credentials

### Running Tests

```bash
# Backend tests
cd src-tauri
cargo test

# Frontend tests (if added)
npm test
```

## Roadmap

- [ ] Complete OAuth flows for Google and GitHub
- [ ] Add data export feature (CSV, JSON)
- [ ] Team/organization accounts
- [ ] Budget limits and alerts
- [ ] Multi-window support
- [ ] Additional providers (Cohere, Together AI, etc.)
- [ ] Mobile app (React Native)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Privacy & Security

- API keys are encrypted using AES-256-GCM before storage
- Keys are stored in your system's secure keychain
- No data is sent to external servers (except provider APIs)
- All data stays local on your machine

## Support

For issues, questions, or feature requests, please open an issue on GitHub.

## Credits

Built with:
- [Tauri](https://tauri.app/) - Desktop app framework
- [React](https://react.dev/) - UI library
- [Tailwind CSS](https://tailwindcss.com/) - Styling
- [SQLx](https://github.com/launchbadge/sqlx) - Database toolkit
- [Recharts](https://recharts.org/) - Charting library
