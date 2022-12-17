**_This is a work in progress._**

# Mirror

Mirror syncs the contents of two directories together across multiple devices. Any files or directories created, modified or removed while Mirror is active will be performed on the rest of the devices.

This project part of an exploration into low-level file syncing, as a stepping stone towards building better [local-first software](https://www.inkandswitch.com/local-first.html) in the future.

## Usage

The following command syncs the given directory to the Mirror installations on each of the devices provided.

```bash
mirror --sync-path ~/some/folder/to/sync \
  --device-addr your.device.com \
  --device-addr another.device.com
```

More options are available through the help command.

```bash
mirror --help
```

## Features

Here are Mirror's intended features.

* **File watching**. Mirror will watch all changes (e.g. file creation, modification and deletion) within the sync directory and propogate them to the other devices.

* **Custom networking protocol**. Data transfer in Mirror will use a custom protocol designed and implemented on top of TCP.

* **Dashboard**. Mirror will include a rudimentary dashboard on the command-line to view upload and download progress as well as presence of other devices.

## Limitations

The following are some missing features from Mirror common. As this project only experimental, here are no plans to add support for these features in the future.

* **Encryption**. The networking protocol does not support encryption as it is out of scope for this project. In the future, an encryption layer can be written in _on top of_ the existing networking protocol.

* **Conflict-resolution**. A conflict-resolution system is tough to design and optimise. Its complexity was decided to be out of scope for this project.

* **File metadata**. Mirror doesn't sync file permissions and other metadata.
