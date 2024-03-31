// const int maxNumber = 400;

// ignore_for_file: constant_identifier_names

enum TaskStatus {
  Pending,
  Running,
  Retrying,
  Success,
  Failure,
  Skipped,
  None;

  // Static method to get enum from int
  static TaskStatus fromInt(int index) {
    if (index < 0 || index >= TaskStatus.values.length) {
      throw ArgumentError('Invalid index for MyEnum: $index');
    }
    return TaskStatus.values[index];
  }

  @override
  String toString() {
    switch (this) {
      case TaskStatus.Pending:
        return "Pending";
      case TaskStatus.Running:
        return "Running";
      case TaskStatus.Retrying:
        return "Retrying";
      case TaskStatus.Success:
        return "Success";
      case TaskStatus.Failure:
        return "Failure";
      case TaskStatus.Skipped:
        return "Skipped";
      case TaskStatus.None:
        return '';
    }
  }
}

extension IntoTaskStatus on int {
  TaskStatus toTaskStatus() => TaskStatus.fromInt(this);
}
