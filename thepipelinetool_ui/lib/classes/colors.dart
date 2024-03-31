import 'package:flutter/material.dart';

import 'task_status.dart';

Color getColorByStatus(TaskStatus taskStatus) {
  return switch (taskStatus) {
    TaskStatus.Pending => Colors.grey,
    TaskStatus.Success => Colors.green,
    TaskStatus.Failure => Colors.red,
    TaskStatus.Running => HexColor.fromHex("#90EE90"),
    TaskStatus.Retrying => Colors.orange,
    TaskStatus.Skipped => const Color.fromARGB(255, 255, 140, 253),
    TaskStatus.None => Colors.transparent,
    // (_) => Colors.transparent,
  };
}

extension HexColor on Color {
  /// String is in the format "aabbcc" or "ffaabbcc" with an optional leading "#".
  static Color fromHex(String hexString) {
    final buffer = StringBuffer();
    if (hexString.length == 6 || hexString.length == 7) buffer.write('ff');
    buffer.write(hexString.replaceFirst('#', ''));
    return Color(int.parse(buffer.toString(), radix: 16));
  }

  /// Prefixes a hash sign if [leadingHashSign] is set to `true` (default is `true`).
  // String toHex({bool leadingHashSign = true}) => '${leadingHashSign ? '#' : ''}'
  //     '${alpha.toRadixString(16).padLeft(2, '0')}'
  //     '${red.toRadixString(16).padLeft(2, '0')}'
  //     '${green.toRadixString(16).padLeft(2, '0')}'
  //     '${blue.toRadixString(16).padLeft(2, '0')}';
}
