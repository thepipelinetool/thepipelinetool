import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../providers/drawer/task_results.dart';
import '../dag_page/dag_page.dart';
import 'drawer.dart';

class Attempts extends ConsumerWidget {
  const Attempts({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final taskResults = ref.watch(taskResultsProvider);

    var results = <Map<String, dynamic>>[];

    switch (taskResults) {
      case AsyncData(:final value):
        results = value;
    }

    return ExpansionPanelList.radio(
      // materialGapSize: 0,
      // dividerColor: Colors.transparent,
      elevation: 0,
      children: [
        for (final value in results)
          ExpansionPanelRadio(
            // backgroundColor: Colors.transparent,
            canTapOnHeader: true,
            headerBuilder: (BuildContext context, bool isExpanded) {
              return ListTile(
                // tileColor: Theme.of(context).primaryColor,

                // style: ListTileStyle.list,
                title: Text(value["attempt"].toString()),
              );
            },
            // body: v["template_args"] == null ? Container() : Container(width: 400, child: jsonView(v["template_args"], false)),
            body: Align(
              alignment: Alignment.topLeft,
              child: Container(
                padding: const EdgeInsets.only(
                    left: kHorizontalPadding,
                    right: kHorizontalPadding,
                    bottom: kHorizontalPadding),
                child: Text(encoder.convert(value["result"])),
              ),
            ),
            value: value["attempt"],
          ),
        // if (v.containsKey("results"))
        // ExpansionPanelRadio(
        //     // backgroundColor: Colors.transparent,

        //     canTapOnHeader: true,
        //     headerBuilder: (BuildContext context, bool isExpanded) =>
        //         const ListTile(title: Text('Attempts')),
        //     body: Align(
        //       alignment: Alignment.topLeft,
        //       child: Container(
        //         padding: const EdgeInsets.only(
        //             left: kHorizontalPadding, right: kHorizontalPadding, bottom: kHorizontalPadding),
        //         child: const Text(""),
        //       ),
        //     ),
        //     value: 'Attempts')
      ],
    );
  }
}
