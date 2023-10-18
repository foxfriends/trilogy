"core::add":
       0: SHIFT  &"core::return"
       5: LOADL  0
      10: SWAP
      11: ADD
      12: RESET
"core::sub":
      13: SHIFT  &"core::return"
      18: LOADL  0
      23: SWAP
      24: SUB
      25: RESET
"core::mul":
      26: SHIFT  &"core::return"
      31: LOADL  0
      36: SWAP
      37: MUL
      38: RESET
"core::div":
      39: SHIFT  &"core::return"
      44: LOADL  0
      49: SWAP
      50: DIV
      51: RESET
"core::intdiv":
      52: SHIFT  &"core::return"
      57: LOADL  0
      62: SWAP
      63: INTDIV
      64: RESET
"core::rem":
      65: SHIFT  &"core::return"
      70: LOADL  0
      75: SWAP
      76: REM
      77: RESET
"core::pow":
      78: SHIFT  &"core::return"
      83: LOADL  0
      88: SWAP
      89: POW
      90: RESET
"core::neg":
      91: NEG
      92: RETURN
"core::glue":
      93: SHIFT  &"core::return"
      98: LOADL  0
     103: SWAP
     104: GLUE
     105: RESET
"core::access":
     106: SHIFT  &"core::return"
     111: LOADL  0
     116: SWAP
     117: ACCESS
     118: RESET
"core::and":
     119: SHIFT  &"core::return"
     124: LOADL  0
     129: SWAP
     130: AND
     131: RESET
"core::or":
     132: SHIFT  &"core::return"
     137: LOADL  0
     142: SWAP
     143: OR
     144: RESET
"core::not":
     145: NOT
     146: RETURN
"core::bitand":
     147: SHIFT  &"core::return"
     152: LOADL  0
     157: SWAP
     158: BITAND
     159: RESET
"core::bitor":
     160: SHIFT  &"core::return"
     165: LOADL  0
     170: SWAP
     171: BITOR
     172: RESET
"core::bitxor":
     173: SHIFT  &"core::return"
     178: LOADL  0
     183: SWAP
     184: BITXOR
     185: RESET
"core::bitneg":
     186: BITNEG
     187: RETURN
"core::lshift":
     188: SHIFT  &"core::return"
     193: LOADL  0
     198: SWAP
     199: BITSHIFTL
     200: RESET
"core::rshift":
     201: SHIFT  &"core::return"
     206: LOADL  0
     211: SWAP
     212: BITSHIFTR
     213: RESET
"core::leq":
     214: SHIFT  &"core::return"
     219: LOADL  0
     224: SWAP
     225: LEQ
     226: RESET
"core::lt":
     227: SHIFT  &"core::return"
     232: LOADL  0
     237: SWAP
     238: LT
     239: RESET
"core::geq":
     240: SHIFT  &"core::return"
     245: LOADL  0
     250: SWAP
     251: GEQ
     252: RESET
"core::gt":
     253: SHIFT  &"core::return"
     258: LOADL  0
     263: SWAP
     264: GT
     265: RESET
"core::refeq":
     266: SHIFT  &"core::return"
     271: LOADL  0
     276: SWAP
     277: REFEQ
     278: RESET
"core::valeq":
     279: SHIFT  &"core::return"
     284: LOADL  0
     289: SWAP
     290: VALEQ
     291: RESET
"core::refneq":
     292: SHIFT  &"core::return"
     297: LOADL  0
     302: SWAP
     303: REFNEQ
     304: RESET
"core::valneq":
     305: SHIFT  &"core::return"
     310: LOADL  0
     315: SWAP
     316: VALNEQ
     317: RESET
"core::pipe":
     318: SHIFT  &"core::return"
     323: LOADL  0
     328: SWAP
     329: CALL   1
     334: RESET
"core::rpipe":
     335: SHIFT  &"core::return"
     340: LOADL  0
     345: CALL   1
     350: RESET
"core::cons":
     351: SHIFT  &"core::return"
     356: LOADL  0
     361: SWAP
     362: CONS
     363: RESET
"core::rcompose":
     364: CLOSE  &"core::return"
     369: CLOSE  &"core::return"
     374: LOADL  0
     379: SWAP
     380: CALL   1
     385: LOADL  1
     390: SWAP
     391: CALL   1
     396: RETURN
"core::compose":
     397: CLOSE  &"core::return"
     402: CLOSE  &"core::return"
     407: LOADL  1
     412: SWAP
     413: CALL   1
     418: LOADL  0
     423: SWAP
     424: CALL   1
     429: RETURN
"core::iter":
     430: COPY
     431: TYPEOF
     432: CONST  "callable"
     437: VALNEQ
     438: JUMPF  &"core::return"
     443: COPY
     444: TYPEOF
     445: CONST  "array"
     450: VALNEQ
     451: JUMPF  &"core::iter_array"
     456: COPY
     457: TYPEOF
     458: CONST  "set"
     463: VALNEQ
     464: JUMPF  &"core::iter_set"
     469: COPY
     470: TYPEOF
     471: CONST  "record"
     476: VALNEQ
     477: JUMPF  &"core::iter_set"
     482: COPY
     483: TYPEOF
     484: CONST  "tuple"
     489: VALNEQ
     490: JUMPF  &"core::iter_list"
     495: COPY
     496: CONST  unit
     501: VALNEQ
     502: JUMPF  &"core::iter_list"
     507: FIZZLE
"core::iter_set":
"core::iter_record":
     508: ENTRIES
"core::iter_array":
     509: CONST  0
     514: CONS
     515: CLOSE  &"core::return"
     520: LOADL  0
     525: UNCONS
     526: COPY
     527: LOADL  1
     532: LENGTH
     533: LT
     534: JUMPF  &"#temp::1::iter_done"
     539: ACCESS
     540: CONST  'next
     545: CONSTRUCT
     546: LOADL  0
     551: UNCONS
     552: CONST  1
     557: ADD
     558: CONS
     559: SETL   0
     564: RETURN
"core::iter_list":
     565: CLOSE  &"core::return"
     570: LOADL  0
     575: COPY
     576: CONST  unit
     581: VALNEQ
     582: JUMPF  &"#temp::1::iter_done"
     587: UNCONS
     588: SETL   0
     593: CONST  'next
     598: CONSTRUCT
     599: RETURN
"#temp::1::iter_done":
     600: CONST  'done
     605: RETURN
"core::reset":
     606: RESET
"core::end":
     607: FIZZLE
"core::return":
     608: RETURN
"core::yield":
     609: LOADR  0
     614: CONST  unit
     619: VALNEQ
     620: JUMPF  &"core::end"
     625: LOADR  0
     630: SWAP
     631: SHIFT  &"#temp::2::yielding"
     636: LOADL  0
     641: SETR   0
     646: RETURN
"#temp::2::yielding":
     647: BECOME 2
"isel#556be4ac8550":
     652: CONST  (unit:0)
     657: CLOSE  &"core::return"
     662: CLOSE  &"core::return"
     667: LOADL  0
     672: UNCONS
     673: COPY
     674: CONST  0
     679: VALEQ
     680: JUMPF  &"#temp::3::next_overload"
     685: POP
     686: COPY
     687: CONST  unit
     692: VALNEQ
     693: JUMPF  &"#temp::5::setup"
"#temp::7::call":
     698: COPY
     699: CALL   0
     704: COPY
     705: CONST  'done
     710: VALEQ
     711: JUMPF  &"#temp::6::end"
     716: POP
     717: POP
     718: JUMP   &"#temp::4::fail"
"#temp::5::setup":
     723: POP
     724: LOADL  1
     729: CONST  0
     734: CONTAINS
     735: JUMPF  &"#temp::8::skip"
     740: LOADL  2
     745: POP
"#temp::8::skip":
     746: LOADL  1
     751: CONST  1
     756: CONTAINS
     757: JUMPF  &"#temp::10::skip"
     762: LOADL  3
     767: CLONE
     768: POP
     769: JUMP   &"#temp::10::skip"
"#temp::12::array_cleanup":
     774: POP
     775: POP
     776: JUMP   &"#temp::9::cleanup"
"#temp::10::skip":
"#temp::13::array_end":
     781: CONST  true
     786: CLOSE  &"#temp::7::call"
     791: LOADL  4
     796: JUMP   &"#temp::14::on_done"
     801: SETL   4
     806: LOADL  1
     811: CONST  0
     816: CONTAINS
     817: JUMPF  &"#temp::15::eval"
     822: LOADL  2
     827: JUMP   &"#temp::16::next"
"#temp::15::eval":
     832: FIZZLE
"#temp::16::next":
     833: LOADL  1
     838: CONST  1
     843: CONTAINS
     844: JUMPF  &"#temp::17::eval"
     849: LOADL  3
     854: JUMP   &"#temp::18::next"
"#temp::17::eval":
     859: CONST  []
"#temp::18::next":
     864: CONST  unit
     869: SWAP
     870: CONS
     871: SWAP
     872: CONS
     873: CONST  'next
     878: CONSTRUCT
     879: RETURN
"#temp::14::on_done":
     880: CONST  'done
     885: RETURN
"#temp::9::cleanup":
"#temp::11::cleanup":
     886: JUMP   &"#temp::4::fail"
"#temp::6::end":
     891: SWAP
     892: CONST  0
     897: CONS
     898: SETL   0
     903: RETURN
"#temp::4::fail":
     904: CONST  (unit:1)
     909: SETL   0
     914: LOADL  0
     919: UNCONS
"#temp::3::next_overload":
     920: COPY
     921: CONST  1
     926: VALEQ
     927: JUMPF  &"#temp::19::next_overload"
     932: POP
     933: COPY
     934: CONST  unit
     939: VALNEQ
     940: JUMPF  &"#temp::21::setup"
"#temp::23::call":
     945: COPY
     946: CALL   0
     951: COPY
     952: CONST  'done
     957: VALEQ
     958: JUMPF  &"#temp::22::end"
     963: POP
     964: POP
     965: JUMP   &"#temp::20::fail"
"#temp::21::setup":
     970: POP
"#var::26::x::0x0000556be4ac5080":
     971: VAR
     972: LOADL  1
     977: CONST  0
     982: CONTAINS
     983: JUMPF  &"#temp::28::assigned"
     988: LOADL  2
     993: COPY
     994: INITL  4
     999: JUMPF  &"#temp::27::compare"
    1004: POP
    1005: JUMP   &"#temp::28::assigned"
"#temp::27::compare":
    1010: LOADL  4
    1015: VALEQ
    1016: JUMPF  &"#temp::25::cleanup"
"#temp::28::assigned":
"#temp::24::skip":
"#var::31::r::0x0000556be4ac1920":
    1021: VAR
"#var::32::y::0x0000556be4aca0c0":
    1022: VAR
    1023: LOADL  1
    1028: CONST  1
    1033: CONTAINS
    1034: JUMPF  &"#temp::34::array_end"
    1039: LOADL  3
    1044: CLONE
    1045: COPY
    1046: CONST  0
    1051: ACCESS
    1052: COPY
    1053: INITL  6
    1058: JUMPF  &"#temp::35::compare"
    1063: POP
    1064: JUMP   &"#temp::36::assigned"
"#temp::35::compare":
    1069: LOADL  6
    1074: VALEQ
    1075: JUMPF  &"#temp::33::array_cleanup"
"#temp::36::assigned":
    1080: CONST  1
    1085: SKIP
    1086: COPY
    1087: INITL  5
    1092: JUMPF  &"#temp::37::compare"
    1097: POP
    1098: JUMP   &"#temp::38::assigned"
"#temp::37::compare":
    1103: LOADL  5
    1108: VALEQ
    1109: JUMPF  &"#temp::30::cleanup"
"#temp::38::assigned":
    1114: JUMP   &"#temp::34::array_end"
"#temp::33::array_cleanup":
    1119: POP
    1120: POP
    1121: JUMP   &"#temp::30::cleanup"
"#temp::34::array_end":
"#temp::29::skip":
    1126: CONST  true
    1131: CONST  false
    1136: CONS
    1137: CONST  unit
    1142: CONS
    1143: CLOSE  &"#temp::23::call"
    1148: LOADL  7
    1153: CONST  false
    1158: SWAP
    1159: UNCONS
    1160: COPY
    1161: CONST  unit
    1166: VALEQ
    1167: SETL   8
    1172: CONST  false
    1177: VALNEQ
    1178: JUMPF  &"#temp::41::alt_second"
    1183: UNCONS
    1184: JUMPF  &"#temp::48::impl_outer"
"#temp::49::impl_inner":
    1189: CONST  false
    1194: SWAP
    1195: JUMPF  &"#temp::47::impl_cleans"
    1200: CONST  true
    1205: CONS
    1206: JUMP   &"#temp::45::impl_out"
"#temp::48::impl_outer":
    1211: CONST  false
    1216: SWAP
    1217: JUMPF  &"#temp::46::impl_cleanf"
    1222: LOADL  6
    1227: LOADL  1
    1232: CONST  4
    1237: CONTAINS
    1238: NOT
    1239: JUMP   &"#temp::50::skip"
    1244: UNSETL 4
"#temp::50::skip":
    1249: COPY
    1250: INITL  4
    1255: JUMPF  &"#temp::51::compare"
    1260: POP
    1261: JUMP   &"#temp::52::assigned"
"#temp::51::compare":
    1266: LOADL  4
    1271: VALEQ
    1272: JUMPF  &"#temp::46::impl_cleanf"
"#temp::52::assigned":
    1277: POP
    1278: CONST  true
    1283: JUMP   &"#temp::49::impl_inner"
"#temp::46::impl_cleanf":
    1288: CONST  false
    1293: CONS
    1294: JUMP   &"#temp::40::alt_maybe"
"#temp::47::impl_cleans":
    1299: CONST  true
    1304: CONS
    1305: JUMP   &"#temp::40::alt_maybe"
"#temp::45::impl_out":
    1310: CONST  true
    1315: CONS
    1316: JUMP   &"#temp::42::alt_out"
"#temp::41::alt_second":
    1321: UNCONS
    1322: JUMPF  &"#temp::53::setup"
"#temp::54::enter":
    1327: COPY
    1328: CALL   0
    1333: COPY
    1334: CONST  'done
    1339: VALNEQ
    1340: JUMPF  &"#temp::55::cleanup"
    1345: DESTRUCT
    1346: CONST  'next
    1351: VALEQ
    1352: JUMPF  &"#temp::55::cleanup"
    1357: UNCONS
    1358: LOADL  1
    1363: CONST  4
    1368: CONTAINS
    1369: NOT
    1370: JUMP   &"#temp::57::skip"
    1375: UNSETL 4
"#temp::57::skip":
    1380: COPY
    1381: INITL  4
    1386: JUMPF  &"#temp::58::compare"
    1391: POP
    1392: JUMP   &"#temp::59::assigned"
"#temp::58::compare":
    1397: LOADL  4
    1402: VALEQ
    1403: JUMPF  &"#temp::55::cleanup"
"#temp::59::assigned":
    1408: UNCONS
    1409: LOADL  1
    1414: CONST  5
    1419: CONTAINS
    1420: NOT
    1421: JUMP   &"#temp::60::skip"
    1426: UNSETL 5
"#temp::60::skip":
    1431: COPY
    1432: INITL  5
    1437: JUMPF  &"#temp::61::compare"
    1442: POP
    1443: JUMP   &"#temp::62::assigned"
"#temp::61::compare":
    1448: LOADL  5
    1453: VALEQ
    1454: JUMPF  &"#temp::55::cleanup"
"#temp::62::assigned":
    1459: POP
    1460: CONST  true
    1465: CONS
    1466: JUMP   &"#temp::56::end"
"#temp::55::cleanup":
    1471: POP
    1472: CONST  true
    1477: CONS
    1478: JUMP   &"#temp::44::alt_cleans"
"#temp::53::setup":
    1483: CONST  [||]
    1488: LOADL  1
    1493: CONST  4
    1498: CONTAINS
    1499: JUMPF  &"#temp::63::nope"
    1504: LOADL  4
    1509: LOADL  10
    1514: CONST  0
    1519: INSERT
    1520: POP
    1521: JUMP   &"#temp::64::next"
"#temp::63::nope":
    1526: CONST  unit
"#temp::64::next":
    1531: LOADL  1
    1536: CONST  5
    1541: CONTAINS
    1542: JUMPF  &"#temp::65::nope"
    1547: LOADL  5
    1552: LOADL  10
    1557: CONST  1
    1562: INSERT
    1563: POP
    1564: JUMP   &"#temp::66::next"
"#temp::65::nope":
    1569: CONST  unit
"#temp::66::next":
    1574: CALL   3
    1579: JUMP   &"#temp::54::enter"
"#temp::56::end":
    1584: CONST  false
    1589: CONS
    1590: JUMP   &"#temp::42::alt_out"
"#temp::40::alt_maybe":
    1595: LOADL  8
    1600: JUMPF  &"#temp::43::alt_cleanf"
    1605: POP
    1606: CONST  &"isel#556be4ac8550"
    1611: CALL   0
    1616: CONST  false
    1621: CONS
    1622: JUMP   &"#temp::41::alt_second"
"#temp::43::alt_cleanf":
    1627: CONST  true
    1632: CONS
    1633: SWAP
    1634: POP
    1635: JUMP   &"#temp::39::on_done"
"#temp::44::alt_cleans":
    1640: CONST  false
    1645: CONS
    1646: SWAP
    1647: POP
    1648: JUMP   &"#temp::39::on_done"
"#temp::42::alt_out":
    1653: SWAP
    1654: POP
    1655: SETL   7
    1660: LOADL  1
    1665: CONST  0
    1670: CONTAINS
    1671: JUMPF  &"#temp::67::eval"
    1676: LOADL  2
    1681: JUMP   &"#temp::68::next"
"#temp::67::eval":
    1686: LOADL  4
"#temp::68::next":
    1691: LOADL  1
    1696: CONST  1
    1701: CONTAINS
    1702: JUMPF  &"#temp::69::eval"
    1707: LOADL  3
    1712: JUMP   &"#temp::70::next"
"#temp::69::eval":
    1717: CONST  []
    1722: LOADL  6
    1727: INSERT
    1728: LOADL  5
    1733: GLUE
"#temp::70::next":
    1734: CONST  unit
    1739: SWAP
    1740: CONS
    1741: SWAP
    1742: CONS
    1743: CONST  'next
    1748: CONSTRUCT
    1749: RETURN
"#temp::39::on_done":
    1750: CONST  'done
    1755: RETURN
"#temp::30::cleanup":
    1756: POP
    1757: POP
"#temp::25::cleanup":
    1758: POP
    1759: JUMP   &"#temp::20::fail"
"#temp::22::end":
    1764: SWAP
    1765: CONST  1
    1770: CONS
    1771: SETL   0
    1776: RETURN
"#temp::20::fail":
    1777: CONST  (unit:2)
    1782: SETL   0
    1787: LOADL  0
    1792: UNCONS
"#temp::19::next_overload":
    1793: CONST  'done
    1798: RETURN
"main#556be4ac5f10":
"trilogy:__entrypoint__":
    1799: CONST  &"isel#556be4ac8550"
    1804: CALL   0
    1809: CONST  false
    1814: CONS
    1815: UNCONS
    1816: JUMPF  &"#temp::73::setup"
"#temp::74::enter":
    1821: COPY
    1822: CALL   0
    1827: COPY
    1828: CONST  'done
    1833: VALNEQ
    1834: JUMPF  &"#temp::75::cleanup"
    1839: DESTRUCT
    1840: CONST  'next
    1845: VALEQ
    1846: JUMPF  &"#temp::75::cleanup"
    1851: UNCONS
    1852: CONST  1
    1857: VALEQ
    1858: JUMPF  &"#temp::75::cleanup"
    1863: UNCONS
    1864: CLONE
    1865: COPY
    1866: CONST  0
    1871: ACCESS
    1872: CONST  1
    1877: VALEQ
    1878: JUMPF  &"#temp::77::array_cleanup"
    1883: CONST  1
    1888: SKIP
    1889: COPY
    1890: CONST  0
    1895: ACCESS
    1896: CONST  2
    1901: VALEQ
    1902: JUMPF  &"#temp::77::array_cleanup"
    1907: CONST  1
    1912: SKIP
    1913: COPY
    1914: CONST  0
    1919: ACCESS
    1920: CONST  3
    1925: VALEQ
    1926: JUMPF  &"#temp::77::array_cleanup"
    1931: CONST  1
    1936: SKIP
    1937: POP
    1938: JUMP   &"#temp::78::array_end"
"#temp::77::array_cleanup":
    1943: POP
    1944: POP
    1945: JUMP   &"#temp::75::cleanup"
"#temp::78::array_end":
    1950: POP
    1951: CONST  true
    1956: CONS
    1957: JUMP   &"#temp::76::end"
"#temp::75::cleanup":
    1962: POP
    1963: CONST  true
    1968: CONS
    1969: JUMP   &"#temp::71::is_fail"
"#temp::73::setup":
    1974: CONST  [|0,1,|]
    1979: CONST  1
    1984: CONST  []
    1989: CONST  1
    1994: INSERT
    1995: CONST  2
    2000: INSERT
    2001: CONST  3
    2006: INSERT
    2007: CALL   3
    2012: JUMP   &"#temp::74::enter"
"#temp::76::end":
    2017: CONST  true
    2022: SLIDE  1
    2027: JUMP   &"#temp::72::is_end"
"#temp::71::is_fail":
    2032: CONST  false
    2037: SLIDE  1
"#temp::72::is_end":
    2042: POP
    2043: EXIT
    2044: CONST  unit
    2049: RETURN

