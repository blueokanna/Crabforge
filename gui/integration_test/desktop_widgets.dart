import 'dart:ffi';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:ffi/ffi.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:win32/win32.dart';

void testDesktopWidgets(String description, WidgetTesterCallback callback) {
  // Request the native accessibility tree before the first test frame. A late
  // Windows request can otherwise receive only a delta from the test binding.
  setUpAll(() async {
    if (!Platform.isWindows) return;
    using((arena) {
      final runner = 'FLUTTER_RUNNER_WIN32_WINDOW'.toNativeUtf16(
        allocator: arena,
      );
      final view = 'FLUTTERVIEW'.toNativeUtf16(allocator: arena);
      final owner = arena<Uint32>();
      var window = 0;
      do {
        window = FindWindowEx(0, window, runner, nullptr);
        if (window == 0) throw StateError('Flutter runner window not found');
        GetWindowThreadProcessId(window, owner);
      } while (owner.value != pid);
      final child = FindWindowEx(window, 0, view, nullptr);
      if (child == 0) throw StateError('Flutter view not found');
      final accessible = DynamicLibrary.open('oleacc.dll')
          .lookupFunction<
            Int32 Function(IntPtr, Uint32, Pointer<GUID>, Pointer<COMObject>),
            int Function(int, int, Pointer<GUID>, Pointer<COMObject>)
          >('AccessibleObjectFromWindow');
      final iid = convertToIID(IID_IUnknown);
      try {
        final result = arena<COMObject>();
        final hr = accessible(child, OBJID_CLIENT.toUnsigned(32), iid, result);
        if (SUCCEEDED(hr) && !result.ref.isNull) {
          final object = IUnknown(result)..detach();
          object.release();
        }
      } finally {
        free(iid);
      }
    });
    final deadline = DateTime.now().add(const Duration(seconds: 5));
    while (!ui.PlatformDispatcher.instance.semanticsEnabled &&
        DateTime.now().isBefore(deadline)) {
      await Future<void>.delayed(const Duration(milliseconds: 25));
    }
    expect(ui.PlatformDispatcher.instance.semanticsEnabled, isTrue);
  });
  testWidgets(description, callback, semanticsEnabled: !Platform.isWindows);
}
