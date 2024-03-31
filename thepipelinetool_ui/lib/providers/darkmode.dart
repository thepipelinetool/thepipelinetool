import 'package:flutter_riverpod/flutter_riverpod.dart';

final darkmodeProvider =
    StateNotifierProvider<DarkMode, bool>((ref) => DarkMode());

class DarkMode extends StateNotifier<bool> {
  DarkMode() : super(false);

  void change(bool text) => state = text;
}
