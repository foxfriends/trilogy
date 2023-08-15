	JUMP &"main"	# 0
"core::add":
	SHIFT &"core::return"	# 5
	LOADL 0	# 10
	SWAP	# 15
	ADD	# 16
	RESET	# 17
"core::sub":
	SHIFT &"core::return"	# 18
	LOADL 0	# 23
	SWAP	# 28
	SUB	# 29
	RESET	# 30
"core::mul":
	SHIFT &"core::return"	# 31
	LOADL 0	# 36
	SWAP	# 41
	MUL	# 42
	RESET	# 43
"core::div":
	SHIFT &"core::return"	# 44
	LOADL 0	# 49
	SWAP	# 54
	DIV	# 55
	RESET	# 56
"core::intdiv":
	SHIFT &"core::return"	# 57
	LOADL 0	# 62
	SWAP	# 67
	INTDIV	# 68
	RESET	# 69
"core::rem":
	SHIFT &"core::return"	# 70
	LOADL 0	# 75
	SWAP	# 80
	REM	# 81
	RESET	# 82
"core::pow":
	SHIFT &"core::return"	# 83
	LOADL 0	# 88
	SWAP	# 93
	POW	# 94
	RESET	# 95
"core::neg":
	NEG	# 96
	RETURN	# 97
"core::glue":
	SHIFT &"core::return"	# 98
	LOADL 0	# 103
	SWAP	# 108
	GLUE	# 109
	RESET	# 110
"core::and":
	SHIFT &"core::return"	# 111
	LOADL 0	# 116
	SWAP	# 121
	AND	# 122
	RESET	# 123
"core::or":
	SHIFT &"core::return"	# 124
	LOADL 0	# 129
	SWAP	# 134
	OR	# 135
	RESET	# 136
"core::not":
	NOT	# 137
	RETURN	# 138
"core::bitand":
	SHIFT &"core::return"	# 139
	LOADL 0	# 144
	SWAP	# 149
	BITAND	# 150
	RESET	# 151
"core::bitor":
	SHIFT &"core::return"	# 152
	LOADL 0	# 157
	SWAP	# 162
	BITOR	# 163
	RESET	# 164
"core::bitxor":
	SHIFT &"core::return"	# 165
	LOADL 0	# 170
	SWAP	# 175
	BITXOR	# 176
	RESET	# 177
"core::bitneg":
	BITNEG	# 178
	RETURN	# 179
"core::lshift":
	SHIFT &"core::return"	# 180
	LOADL 0	# 185
	SWAP	# 190
	BITSHIFTL	# 191
	RESET	# 192
"core::rshift":
	SHIFT &"core::return"	# 193
	LOADL 0	# 198
	SWAP	# 203
	BITSHIFTR	# 204
	RESET	# 205
"core::leq":
	SHIFT &"core::return"	# 206
	LOADL 0	# 211
	SWAP	# 216
	LEQ	# 217
	RESET	# 218
"core::lt":
	SHIFT &"core::return"	# 219
	LOADL 0	# 224
	SWAP	# 229
	LT	# 230
	RESET	# 231
"core::geq":
	SHIFT &"core::return"	# 232
	LOADL 0	# 237
	SWAP	# 242
	GEQ	# 243
	RESET	# 244
"core::gt":
	SHIFT &"core::return"	# 245
	LOADL 0	# 250
	SWAP	# 255
	GT	# 256
	RESET	# 257
"core::refeq":
	SHIFT &"core::return"	# 258
	LOADL 0	# 263
	SWAP	# 268
	REFEQ	# 269
	RESET	# 270
"core::valeq":
	SHIFT &"core::return"	# 271
	LOADL 0	# 276
	SWAP	# 281
	VALEQ	# 282
	RESET	# 283
"core::refneq":
	SHIFT &"core::return"	# 284
	LOADL 0	# 289
	SWAP	# 294
	REFNEQ	# 295
	RESET	# 296
"core::valneq":
	SHIFT &"core::return"	# 297
	LOADL 0	# 302
	SWAP	# 307
	VALNEQ	# 308
	RESET	# 309
"core::pipe":
	SHIFT &"core::return"	# 310
	LOADL 0	# 315
	SWAP	# 320
	CALL 1	# 321
	RESET	# 326
"core::rpipe":
	SHIFT &"core::return"	# 327
	LOADL 0	# 332
	CALL 1	# 337
	RESET	# 342
"core::cons":
	SHIFT &"core::return"	# 343
	LOADL 0	# 348
	SWAP	# 353
	CONS	# 354
	RESET	# 355
"core::rcompose":
	CLOSE &"core::return"	# 356
	CLOSE &"core::return"	# 361
	LOADL 0	# 366
	SWAP	# 371
	CALL 1	# 372
	LOADL 1	# 377
	SWAP	# 382
	CALL 1	# 383
	RETURN	# 388
"core::compose":
	CLOSE &"core::return"	# 389
	CLOSE &"core::return"	# 394
	LOADL 1	# 399
	SWAP	# 404
	CALL 1	# 405
	LOADL 0	# 410
	SWAP	# 415
	CALL 1	# 416
	RETURN	# 421
"core::iter":
	COPY	# 422
	TYPEOF	# 423
	CONST "callable"	# 424
	VALNEQ	# 429
	JUMPF &"core::return"	# 430
	COPY	# 435
	TYPEOF	# 436
	CONST "array"	# 437
	VALNEQ	# 442
	JUMPF &"core::iter_array"	# 443
	COPY	# 448
	TYPEOF	# 449
	CONST "set"	# 450
	VALNEQ	# 455
	JUMPF &"core::iter_record"	# 456
	COPY	# 461
	TYPEOF	# 462
	CONST "record"	# 463
	VALNEQ	# 468
	JUMPF &"core::iter_record"	# 469
	COPY	# 474
	TYPEOF	# 475
	CONST "tuple"	# 476
	VALNEQ	# 481
	JUMPF &"core::iter_list"	# 482
	COPY	# 487
	CONST unit	# 488
	VALNEQ	# 493
	JUMPF &"core::iter_list"	# 494
	FIZZLE	# 499
"core::iter_record":
"core::iter_set":
	ENTRIES	# 500
"core::iter_array":
	CONST 0+0i	# 501
	CONS	# 506
	CLOSE &"core::return"	# 507
	LOADL 0	# 512
	UNCONS	# 517
	COPY	# 518
	LOADL 1	# 519
	LENGTH	# 524
	LT	# 525
	JUMPF &"#temp::1::iter_done"	# 526
	ACCESS	# 531
	CONST 'next	# 532
	SWAP	# 537
	CONSTRUCT	# 538
	LOADL 0	# 539
	UNCONS	# 544
	CONST 1+0i	# 545
	ADD	# 550
	CONS	# 551
	SETL 0	# 552
	RETURN	# 557
"core::iter_list":
	CLOSE &"core::return"	# 558
	LOADL 0	# 563
	COPY	# 568
	CONST unit	# 569
	VALNEQ	# 574
	JUMPF &"#temp::1::iter_done"	# 575
	UNCONS	# 580
	SETL 0	# 581
	CONST 'next	# 586
	SWAP	# 591
	CONSTRUCT	# 592
	RETURN	# 593
"#temp::1::iter_done":
	CONST 'done	# 594
	RETURN	# 599
"core::reset":
	RESET	# 600
"core::end":
	FIZZLE	# 601
"core::return":
	RETURN	# 602
"core::yield":
	LOADR 0	# 603
	CONST unit	# 608
	VALNEQ	# 613
	RJUMPF &"core::end"	# 614
	LOADR 0	# 619
	SWAP	# 624
	SHIFT &"#temp::2::yielding"	# 625
	LOADL 0	# 630
	SETR 0	# 635
	RETURN	# 640
"#temp::2::yielding":
	BECOME 2	# 641
"is_ok#6000000d4fd0":
"file:///Users/cam/code/personal/trilogy/samples/rule.tri#$is_ok":
"file:///Users/cam/code/personal/trilogy/samples/rule.tri":
	CONST (unit:0+0i)	# 646
	RCLOSE &"core::return"	# 651
	LOADL 0	# 656
	UNCONS	# 661
	COPY	# 662
	CONST 0+0i	# 663
	VALEQ	# 668
	JUMPF &"#temp::3::skip"	# 669
	POP	# 674
	COPY	# 675
	CONST unit	# 676
	VALNEQ	# 681
	JUMPF &"#temp::5::setup"	# 682
"#temp::7::call":
	COPY	# 687
	CALL 0	# 688
	COPY	# 693
	CONST 'done	# 694
	VALEQ	# 699
	JUMPF &"#temp::4::fail"	# 700
	JUMP &"#temp::6::end"	# 705
"#temp::5::setup":
	POP	# 710
	LOADL 1	# 711
	CONST 0+0i	# 716
	CONTAINS	# 721
	JUMPF &"#temp::8::skip"	# 722
	LOADL 2	# 727
	CONST 1+0i	# 732
	VALEQ	# 737
	JUMPF &"#temp::9::cleanup"	# 738
"#temp::8::skip":
	JUMP &"#temp::10::close"	# 743
"#temp::9::cleanup":
	JUMP &"#temp::4::fail"	# 748
"#temp::10::close":
	CONST true	# 753
	RCLOSE &"#temp::7::call"	# 758
	CONST false	# 763
	SWAP	# 768
	JUMPF &"#temp::11::on_done"	# 769
	CONST 1+0i	# 774
	CONST unit	# 779
	SWAP	# 784
	CONS	# 785
	CONST 'next	# 786
	SWAP	# 791
	CONSTRUCT	# 792
	RETURN	# 793
"#temp::11::on_done":
	CONST 'done	# 794
	RETURN	# 799
"#temp::6::end":
	SWAP	# 800
	CONST 0+0i	# 801
	CONS	# 806
	SETL 0	# 807
	RETURN	# 812
"#temp::4::fail":
	POP	# 813
	CONST (unit:1+0i)	# 814
	SETL 0	# 819
"#temp::3::skip":
	COPY	# 824
	CONST 1+0i	# 825
	VALEQ	# 830
	JUMPF &"#temp::12::skip"	# 831
	POP	# 836
	COPY	# 837
	CONST unit	# 838
	VALNEQ	# 843
	JUMPF &"#temp::14::setup"	# 844
"#temp::16::call":
	COPY	# 849
	CALL 0	# 850
	COPY	# 855
	CONST 'done	# 856
	VALEQ	# 861
	JUMPF &"#temp::13::fail"	# 862
	JUMP &"#temp::15::end"	# 867
"#temp::14::setup":
	POP	# 872
	LOADL 1	# 873
	CONST 0+0i	# 878
	CONTAINS	# 883
	JUMPF &"#temp::17::skip"	# 884
	LOADL 2	# 889
	CONST 2+0i	# 894
	VALEQ	# 899
	JUMPF &"#temp::18::cleanup"	# 900
"#temp::17::skip":
	JUMP &"#temp::19::close"	# 905
"#temp::18::cleanup":
	JUMP &"#temp::13::fail"	# 910
"#temp::19::close":
	CONST true	# 915
	RCLOSE &"#temp::16::call"	# 920
	CONST false	# 925
	SWAP	# 930
	JUMPF &"#temp::20::on_done"	# 931
	CONST 2+0i	# 936
	CONST unit	# 941
	SWAP	# 946
	CONS	# 947
	CONST 'next	# 948
	SWAP	# 953
	CONSTRUCT	# 954
	RETURN	# 955
"#temp::20::on_done":
	CONST 'done	# 956
	RETURN	# 961
"#temp::15::end":
	SWAP	# 962
	CONST 1+0i	# 963
	CONS	# 968
	SETL 0	# 969
	RETURN	# 974
"#temp::13::fail":
	POP	# 975
	CONST (unit:2+0i)	# 976
	SETL 0	# 981
"#temp::12::skip":
	COPY	# 986
	CONST 2+0i	# 987
	VALEQ	# 992
	JUMPF &"#temp::21::skip"	# 993
	POP	# 998
	COPY	# 999
	CONST unit	# 1000
	VALNEQ	# 1005
	JUMPF &"#temp::23::setup"	# 1006
"#temp::25::call":
	COPY	# 1011
	CALL 0	# 1012
	COPY	# 1017
	CONST 'done	# 1018
	VALEQ	# 1023
	JUMPF &"#temp::22::fail"	# 1024
	JUMP &"#temp::24::end"	# 1029
"#temp::23::setup":
	POP	# 1034
	LOADL 1	# 1035
	CONST 0+0i	# 1040
	CONTAINS	# 1045
	JUMPF &"#temp::26::skip"	# 1046
	LOADL 2	# 1051
	CONST 3+0i	# 1056
	VALEQ	# 1061
	JUMPF &"#temp::27::cleanup"	# 1062
"#temp::26::skip":
	JUMP &"#temp::28::close"	# 1067
"#temp::27::cleanup":
	JUMP &"#temp::22::fail"	# 1072
"#temp::28::close":
	CONST true	# 1077
	RCLOSE &"#temp::25::call"	# 1082
	CONST false	# 1087
	SWAP	# 1092
	JUMPF &"#temp::29::on_done"	# 1093
	CONST 3+0i	# 1098
	CONST unit	# 1103
	SWAP	# 1108
	CONS	# 1109
	CONST 'next	# 1110
	SWAP	# 1115
	CONSTRUCT	# 1116
	RETURN	# 1117
"#temp::29::on_done":
	CONST 'done	# 1118
	RETURN	# 1123
"#temp::24::end":
	SWAP	# 1124
	CONST 2+0i	# 1125
	CONS	# 1130
	SETL 0	# 1131
	RETURN	# 1136
"#temp::22::fail":
	POP	# 1137
	CONST (unit:3+0i)	# 1138
	SETL 0	# 1143
"#temp::21::skip":
	CONST 'done	# 1148
	RETURN	# 1153
"main":
"main#6000000d5000":
"file:///Users/cam/code/personal/trilogy/samples/rule.tri#$main":
	VAR	# 1154
	CONST true	# 1155
"#temp::30::let":
	CONST false	# 1160
	SWAP	# 1165
	RJUMPF &"core::end"	# 1166
	CONST 0+0i	# 1171
	UNSETL 0	# 1176
	COPY	# 1181
	INITL 0	# 1182
	JUMPF &"#temp::31::compare"	# 1187
	POP	# 1192
	JUMP &"#temp::32::assigned"	# 1193
"#temp::31::compare":
	LOADL 0	# 1198
	VALEQ	# 1203
	RJUMPF &"core::end"	# 1204
"#temp::32::assigned":
	CONST false	# 1209
	CLOSE &"#temp::33::iter"	# 1214
	VAR	# 1219
	VAR	# 1220
	LOADL 3	# 1221
	CALL 0	# 1226
	CONST false	# 1231
	CONS	# 1236
	CLOSE &"#temp::35::iter_end"	# 1237
	LOADL 5	# 1242
	UNCONS	# 1247
	JUMPF &"#temp::36::setup"	# 1248
"#temp::37::enter":
	COPY	# 1253
	CALL 0	# 1254
	COPY	# 1259
	CONST 'done	# 1260
	VALEQ	# 1265
	JUMPF &"#temp::38::cleanup"	# 1266
	DESTRUCT	# 1271
	SWAP	# 1272
	CONST 'next	# 1273
	VALEQ	# 1278
	JUMPF &"#temp::38::cleanup"	# 1279
	UNCONS	# 1284
	COPY	# 1285
	INITL 4	# 1286
	JUMPF &"#temp::40::compare"	# 1291
	POP	# 1296
	JUMP &"#temp::41::assigned"	# 1297
"#temp::40::compare":
	LOADL 4	# 1302
	VALEQ	# 1307
	JUMPF &"#temp::38::cleanup"	# 1308
"#temp::41::assigned":
	POP	# 1313
	CONST true	# 1314
	CONS	# 1319
	JUMP &"#temp::39::end"	# 1320
"#temp::38::cleanup":
	POP	# 1325
	CONST true	# 1326
	CONS	# 1331
	JUMP &"#temp::34::iter_out"	# 1332
"#temp::36::setup":
	CONST [||]	# 1337
	CONST unit	# 1342
	CALL 2	# 1347
	RJUMP &"#temp::37::enter"	# 1352
"#temp::39::end":
	SETL 5	# 1357
	LOADL 0	# 1362
	LOADL 4	# 1367
	ADD	# 1372
	COPY	# 1373
	SETL 0	# 1374
	CONST 'next	# 1379
	SWAP	# 1384
	CONSTRUCT	# 1385
	RETURN	# 1386
"#temp::34::iter_out":
	CONST 'done	# 1387
"#temp::35::iter_end":
	RETURN	# 1392
"#temp::33::iter":
	CALL 0	# 1393
"#temp::42::for":
	COPY	# 1398
	CALL 0	# 1399
	COPY	# 1404
	CONST 'done	# 1405
	VALNEQ	# 1410
	JUMPF &"#temp::43::for_end"	# 1411
	COPY	# 1416
	TYPEOF	# 1417
	CONST "struct"	# 1418
	VALEQ	# 1423
	RJUMPF &"core::end"	# 1424
	DESTRUCT	# 1429
	SWAP	# 1430
	CONST 'next	# 1431
	VALEQ	# 1436
	RJUMPF &"core::end"	# 1437
	CONST true	# 1442
	SETL 2	# 1447
	POP	# 1452
	RJUMP &"#temp::42::for"	# 1453
"#temp::43::for_end":
	POP	# 1458
	POP	# 1459
	POP	# 1460
	LOADL 0	# 1461
	EXIT	# 1466
	POP	# 1467
	POP	# 1468
	CONST unit	# 1469
	RETURN	# 1474
