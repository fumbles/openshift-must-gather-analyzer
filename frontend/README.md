# Camgi Frontend

Modern React + Tailwind CSS frontend for the must-gather explorer.

## Development

\`\`\`bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
\`\`\`

## Build Output

The build process generates:
- \`dist/assets/index.js\` - Single minified JavaScript bundle
- \`dist/assets/index.css\` - Single minified CSS bundle

These files are embedded in the Rust binary at compile time.

## Design System

The frontend uses a yamlwrangler-inspired dark theme with:
- Slate backgrounds (#020617, #0f172a, #1e293b)
- Red accents (#ef4444)
- Rounded corners (12px-24px)
- Subtle shadows and borders
