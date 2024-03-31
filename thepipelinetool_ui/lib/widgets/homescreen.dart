// ignore_for_file: constant_identifier_names

import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:thepipelinetool/classes/dag_info.dart';
import 'package:thepipelinetool/widgets/appbar.dart';
import 'package:thepipelinetool/classes/dag_options.dart';
import 'package:thepipelinetool/providers/dags.dart';
// import 'package:thepipelinetool/homescreen_row.dart';
import 'package:http/http.dart' as http;

import '../main.dart';
import 'dag_page/dag_page.dart';
import 'dag_page/task_view/table_cell.dart';

class DagLink extends ConsumerWidget {
  final DagInfo info;
  const DagLink({super.key, required this.info});

  @override
  Widget build(BuildContext context, WidgetRef ref) => MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          onTap: () {
            // handle the tap event
            FocusScope.of(context).requestFocus(FocusNode());

            // ref.invalidate(selectedItemProvider(dagName));
            context.goNamed('dag', pathParameters: {'dag_name': info.dagName});
          },
          child: Text(
            info.dagName,
            // style: const TextStyle(
            //     decoration: TextDecoration.underline), // optional
          ),
        ),
      );
}

class DTS extends DataTableSource {
  final List<DagInfo> all;
  DTS(this.all);

  @override
  DataRow getRow(int index) {
    return DataRow.byIndex(
      index: index,
      cells: [
        DataCell(DagLink(info: all[index])),
        DataCell(
          SizedBox(
            width: 200,
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                for (int i = 0; i < 10; i++)
                  Padding(
                    padding:
                        i == 0 ? EdgeInsets.zero : EdgeInsets.only(left: 2),
                    child: GestureDetector(
                      onTap: () async {
                        // String url =
                        //     "${Config.BASE_URL}${Config.TRIGGER}${all[index].dagName}";
                        // final response = await http.get(Uri.parse(url));
                      },
                      child: MouseRegion(
                        cursor: SystemMouseCursors.click,
                        child: Container(
                          color: Colors.green,
                          width: cellWidth,
                          height: i * 1.0, // TODO replace with elapsed time
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
        DataCell(Text(all[index].options.schedule ?? '')),

        // Text(allDagOptions[index].dagName)),
        DataCell(Text(all[index].lastRun?.toIso8601String() ?? '')),
        DataCell(Text(all[index].nextRun?.toIso8601String() ?? '')),
        DataCell(
          GestureDetector(
            onTap: () async {
              String url =
                  "${Config.BASE_URL}${Config.TRIGGER}${all[index].dagName}";
              final response = await http.get(Uri.parse(url));
            },
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: Icon(Icons.play_arrow),
            ),
          ),
        ),
        // DataCell(Text('#cel4$index')),
        // DataCell(Text('#cel5$index')),
        // DataCell(Text('#cel6$index')),
      ],
    );
  }

  @override
  bool get isRowCountApproximate => false;

  @override
  int get rowCount => all.length;

  @override
  int get selectedRowCount => 0;
}

class HomeScreen extends ConsumerStatefulWidget {
  // final Widget Function(BuildContext context) bottomBar;

  const HomeScreen({Key? key}) : super(key: key);
  @override
  HomeScreenState createState() => HomeScreenState();
}

enum Columns {
  DAG,
  // endDate;
  // maxAttempts;
  // retryDelay;
  Runs,
  Schedule,
  Last_Run,
  Next_Run,
  Actions,
  // startDate;
  // timeout;
}

class HomeScreenState extends ConsumerState<HomeScreen> {
  HomeScreenState();
  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  final ScrollController _scrollController = ScrollController();

  int sortColumn = 0;
  bool ascending = false;

  @override
  Widget build(BuildContext context) {
    final dagProvider = ref.watch(dagsProvider);

    // return Container();

    // final data = {
    //   "name": "Aleix Melon",
    //   "id": "E00245",
    //   "role": ["Dev", "DBA"],
    //   "age": 23,
    //   "doj": "11-12-2019",
    //   "married": false,
    //   "address": {
    //     "street": "32, Laham St.",
    //     "city": "Innsbruck",
    //     "country": "Austria"
    //   },
    //   "referred-by": "E0012"
    // };

    return Scaffold(
      appBar: AppBar(
          scrolledUnderElevation: 0,
          toolbarHeight: kMyToolbarHeight,
          title: const MyAppBar()),
      // backgroundColor: Colors.red,
      body:

          //   Scrollbar(
          // child:
          PaginatedDataTable2(
        wrapInCard: false,
        // controller: _scrollController,
        sortColumnIndex: sortColumn,
        sortAscending: ascending,
        columns: Columns.values
            .map((e) => DataColumn(
                  label: Text(e.name.replaceFirst('_', ' ')),
                  onSort: ![Columns.DAG].contains(e)
                      ? null
                      : // TODO more sortable columns?
                      (int columnIndex, bool ascending_) {
                          setState(() {
                            sortColumn = columnIndex;
                            ascending = ascending_;
                          });
                        },
                ))
            .toList(),
        source:
            // children: <Widget>[
            //   const SliverPadding(
            //     padding: EdgeInsets.only(bottom: 8.0),
            //     sliver: SliverAppBar(
            //       backgroundColor: Color.fromARGB(255, 170, 170, 170),
            //       pinned: true,
            //       toolbarHeight: 40,
            //       title: Row(
            //         children: [Text('DAG')],
            //       ),
            //     ),
            //   ),
            switch (dagProvider) {
          AsyncData(:final value) => DTS(value
            ..sort((a, b) {
              if (ascending) {
                [a, b] = [b, a];
              }
              // print(ascending);

              switch (Columns.values[sortColumn]) {
                case Columns.DAG:
                  return a.dagName.compareTo(b.dagName);
                case Columns.Schedule:
                  if (a.options.schedule != null &&
                      b.options.schedule != null) {
                    return a.options.schedule!.compareTo(b.options.schedule!);
                  }

                  if (b.options.schedule == null) {
                    return -1;
                  }

                  return 1;
                default:
                  return a.dagName.compareTo(b.dagName);
                // case Columns.Last_Run:
                //   // TODO

                // case Columns.Next_Run:
                //   // TODO
                //   return a.dagName.compareTo(b.dagName);
              }
            })),
          // SliverList.separated(
          //     itemCount: value.length,
          //     itemBuilder: (BuildContext context, int index) {
          //       return HomeScreenRow(dagOptions: value[index]);
          //     },
          //     separatorBuilder: (BuildContext context, int index) =>
          //         const Divider(),
          //   ),
          AsyncError(:final error) => () {
              print(error);
              return DTS([]);
            }(),
          _ => DTS([]),
        },
        // ],
        // ),
        // ),
      ),
      //   ),
      // ),
    );
  }
}
