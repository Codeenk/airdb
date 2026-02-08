#!/bin/bash
# Commit and tag v0.2.1 release

set -e

VERSION="0.2.1"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$PROJECT_ROOT"

echo "🚀 Preparing v$VERSION for commit"

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "📝 Staging changes..."
    
    # Add all changes
    git add -A
    
    # Show what will be committed
    echo ""
    echo "Files to be committed:"
    git status --short
    echo ""
    
    read -p "Commit these changes? (y/N) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Commit with detailed message
        git commit -m "Release v$VERSION - Productization Layer

✨ Features:
- Auto-start at boot (Windows/Linux/macOS)
- System-wide CLI PATH installation
- Enhanced terminal UX with colors and formatting
- Visual SQL table editor with migration preview
- Complete updater UI in Settings
- Global update banner

🔧 Improvements:
- Bootstrapper path resolution
- Update state management
- Migration preview formatting
- Error messages

📚 Documentation:
- Added team-workflows.md
- Created CHANGELOG.md
- Created RELEASE_NOTES.md
- Created release scripts

🐛 Fixes:
- Auto-start now uses bootstrapper
- CLI PATH respects platform conventions
- Update checks non-blocking during operations

Dependencies:
- Added: which, colored, winreg (Windows)
- New modules: installer, cli/formatter"
        
        echo "✓ Changes committed"
    else
        echo "Aborted."
        exit 1
    fi
else
    echo "✓ No uncommitted changes"
fi

# Create and push tag
read -p "Create and push tag v$VERSION? (y/N) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Create annotated tag
    git tag -a "v$VERSION" -m "AirDB v$VERSION - Productization Release

This release makes AirDB production-ready with:
- Cross-platform auto-start
- System-wide CLI access
- Professional terminal UX
- Visual table editor
- Complete updater UI

See CHANGELOG.md for full details."
    
    echo "✓ Tag v$VERSION created"
    
    # Push to remote
    read -p "Push to origin? (y/N) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push origin main
        git push origin "v$VERSION"
        echo "✓ Pushed to origin"
        echo ""
        echo "🎉 v$VERSION is now ready for release!"
        echo ""
        echo "Next steps:"
        echo "  1. Run ./scripts/release.sh to build binaries"
        echo "  2. Create GitHub release with release/0.2.1/ artifacts"
        echo "  3. Upload RELEASE_NOTES.md as release description"
    fi
else
    echo "Tag not created."
fi

echo ""
echo "Done!"
