AirDB - Local-First, GitHub-Backed Database Platform
=====================================================

Quick Installation
------------------

1. Extract this archive:
   tar -xzf airdb-*-linux-x64.tar.gz

2. Run the installer:
   cd airdb-*-linux-x64
   ./install.sh

   Or for system-wide installation (requires sudo):
   sudo ./install.sh

3. Verify installation:
   airdb --version

Usage
-----

CLI Commands:
  airdb init <name>     Create a new database project
  airdb serve           Start the API server
  airdb status          Show project status
  airdb update check    Check for updates

GUI Application:
  airdb-desktop         Launch the graphical interface
  
Or find "AirDB" in your application menu.

Uninstallation
--------------

Run the uninstall script:
  ~/.local/share/airdb/uninstall.sh

Or if installed system-wide:
  sudo /opt/airdb/uninstall.sh

Documentation
-------------

Full documentation: https://github.com/Codeenk/airdb/tree/main/docs

Support
-------

Report issues: https://github.com/Codeenk/airdb/issues
