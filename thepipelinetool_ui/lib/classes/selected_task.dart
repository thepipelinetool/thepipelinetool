class SelectedTask {
  final String dagName;
  final String runId;
  final String taskId;

  SelectedTask(
      {required this.runId, required this.taskId, required this.dagName});
}
