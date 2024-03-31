import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:thepipelinetool/classes/selected_task.dart';

// class SelectedTaskStateNotifier extends StateNotifier<SelectedTask?> {
//   SelectedTaskStateNotifier() : super(null);

//   void updateData(SelectedTask newData) => state = newData;
// }

final selectedTaskProvider = StateProvider<SelectedTask?>((ref) => null);
