---
title: "Documentation"
description: "Install, use and develop NARF."
icon: "menu_book"
weight: 1
---

NARF rebuilds a NixOS configuration flake and records every rebuild as a git
commit; it is meant for NixOS users who manage their systems with a flake in a
git repository. A plain `nixos-rebuild` leaves no trace in the configuration
repository. NARF wraps update, rebuild and commit into one step, so the git
history of the flake doubles as a log of every generation built on each host.
