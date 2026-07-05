import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

final class NMLCoreHandle extends Opaque {}

typedef _NmlInitNative = Pointer<NMLCoreHandle> Function(Pointer<Utf8> configPath);
typedef _NmlShutdownNative = Void Function(Pointer<NMLCoreHandle> handle);
typedef _NmlVersionNative = Pointer<Utf8> Function();
typedef _NmlLaunchNative = Int32 Function(
  Pointer<NMLCoreHandle> handle,
  Pointer<Utf8> versionId,
  Pointer<Utf8> playerName,
  Bool isOffline,
);

typedef _NmlInit = Pointer<NMLCoreHandle> Function(Pointer<Utf8> configPath);
typedef _NmlShutdown = void Function(Pointer<NMLCoreHandle> handle);
typedef _NmlVersion = Pointer<Utf8> Function();
typedef _NmlLaunch = int Function(
  Pointer<NMLCoreHandle> handle,
  Pointer<Utf8> versionId,
  Pointer<Utf8> playerName,
  bool isOffline,
);

class NMLCore {
  NMLCore._(this._lib)
      : _init = _lib.lookupFunction<_NmlInitNative, _NmlInit>('nml_init'),
        _shutdown = _lib.lookupFunction<_NmlShutdownNative, _NmlShutdown>('nml_shutdown'),
        _version = _lib.lookupFunction<_NmlVersionNative, _NmlVersion>('nml_version'),
        _launch = _lib.lookupFunction<_NmlLaunchNative, _NmlLaunch>('nml_launch');

  final DynamicLibrary _lib;
  final _NmlInit _init;
  final _NmlShutdown _shutdown;
  final _NmlVersion _version;
  final _NmlLaunch _launch;

  Pointer<NMLCoreHandle>? _handle;

  static NMLCore load() {
    final libraryPath = Platform.isWindows ? 'nml_core.dll' : 'libnml_core.so';
    return NMLCore._(DynamicLibrary.open(libraryPath));
  }

  bool init({String? configPath}) {
    final config = configPath?.toNativeUtf8() ?? nullptr;
    try {
      final handle = _init(config);
      if (handle == nullptr) return false;
      _handle = handle;
      return true;
    } finally {
      if (config != nullptr) malloc.free(config);
    }
  }

  String version() {
    final ptr = _version();
    if (ptr == nullptr) return 'unknown';
    return ptr.toDartString();
  }

  int launch({required String versionId, required String playerName, bool offline = true}) {
    final handle = _handle;
    if (handle == null || handle == nullptr) return -1;

    final version = versionId.toNativeUtf8();
    final player = playerName.toNativeUtf8();
    try {
      return _launch(handle, version, player, offline);
    } finally {
      malloc.free(version);
      malloc.free(player);
    }
  }

  void dispose() {
    final handle = _handle;
    if (handle != null && handle != nullptr) {
      _shutdown(handle);
      _handle = null;
    }
  }
}
