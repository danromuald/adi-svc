# adi-web

Beautiful React-based testing UI for adi-svc (Azure AI Document Intelligence Service).

## Features

- 🎨 Modern, responsive design with gradient background
- 📤 Support for both URL-based and file upload analysis
- 🔄 Real-time polling for analysis results
- 📊 Multiple model types (Read, Layout, Invoice, Receipt, ID Document, Business Card, W-2)
- 💅 Beautiful visualizations of analysis results
- 📱 Mobile-friendly responsive design

## Prerequisites

- Node.js 18+
- Running adi-svc backend on `http://localhost:8080`

## Installation

```bash
npm install
```

## Running

```bash
npm start
```

The app will open at `http://localhost:3000` and proxy API requests to `http://localhost:8080`.

## Usage

1. **Select a Model**: Choose from Read, Layout, Invoice, Receipt, ID Document, Business Card, or W-2
2. **Choose Input Method**:
   - **URL**: Enter a publicly accessible document URL
   - **Upload**: Drag & drop or click to select a local file
3. **Analyze**: Click the "Analyze Document" button
4. **View Results**: See extracted text, page information, and table data

## Supported File Types

- PDF (`.pdf`)
- Images (`.png`, `.jpg`, `.jpeg`, `.tiff`, `.bmp`)

## Environment Variables

Create a `.env` file to customize the API endpoint:

```
REACT_APP_API_BASE=http://localhost:8080/api/v1
```

## Build for Production

```bash
npm run build
```

This creates an optimized production build in the `build/` directory.

