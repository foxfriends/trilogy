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
"isel#56343246b550":
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
     698: JUMP   &"#temp::8::call"
"#temp::7::precall":
     703: SWAP
     704: POP
"#temp::8::call":
     705: COPY
     706: CALL   0
     711: COPY
     712: CONST  'done
     717: VALEQ
     718: JUMPF  &"#temp::6::end"
     723: POP
     724: POP
     725: JUMP   &"#temp::4::fail"
"#temp::5::setup":
     730: POP
     731: LOADL  1
     736: CONST  0
     741: CONTAINS
     742: JUMPF  &"#temp::9::skip"
     747: LOADL  2
     752: POP
"#temp::9::skip":
     753: LOADL  1
     758: CONST  1
     763: CONTAINS
     764: JUMPF  &"#temp::14::array_end"
     769: LOADL  3
     774: CLONE
     775: POP
     776: JUMP   &"#temp::14::array_end"
"#temp::13::array_cleanup":
     781: POP
     782: POP
     783: JUMP   &"#temp::10::cleanup"
"#temp::14::array_end":
"#temp::11::skip":
     788: CONST  true
     793: CLOSE  &"#temp::7::precall"
     798: LOADL  4
     803: JUMP   &"#temp::15::on_done"
     808: SETL   4
     813: LOADL  1
     818: CONST  0
     823: CONTAINS
     824: JUMPF  &"#temp::16::eval"
     829: LOADL  2
     834: JUMP   &"#temp::17::next"
"#temp::16::eval":
     839: FIZZLE
"#temp::17::next":
     840: LOADL  1
     845: CONST  1
     850: CONTAINS
     851: JUMPF  &"#temp::18::eval"
     856: LOADL  3
     861: JUMP   &"#temp::19::next"
"#temp::18::eval":
     866: CONST  []
"#temp::19::next":
     871: CONST  unit
     876: SWAP
     877: CONS
     878: SWAP
     879: CONS
     880: CONST  'next
     885: CONSTRUCT
     886: RETURN
"#temp::15::on_done":
     887: CONST  'done
     892: RETURN
"#temp::10::cleanup":
"#temp::12::cleanup":
     893: JUMP   &"#temp::4::fail"
"#temp::6::end":
     898: SWAP
     899: CONST  0
     904: CONS
     905: SETL   0
     910: RETURN
"#temp::4::fail":
     911: CONST  (unit:1)
     916: SETL   0
     921: LOADL  0
     926: UNCONS
"#temp::3::next_overload":
     927: COPY
     928: CONST  1
     933: VALEQ
     934: JUMPF  &"#temp::20::next_overload"
     939: POP
     940: COPY
     941: CONST  unit
     946: VALNEQ
     947: JUMPF  &"#temp::22::setup"
     952: JUMP   &"#temp::25::call"
"#temp::24::precall":
     957: SWAP
     958: POP
"#temp::25::call":
     959: COPY
     960: CALL   0
     965: COPY
     966: CONST  'done
     971: VALEQ
     972: JUMPF  &"#temp::23::end"
     977: POP
     978: POP
     979: JUMP   &"#temp::21::fail"
"#temp::22::setup":
     984: POP
"#var::28::x::0x0000563432468080":
     985: VAR
     986: LOADL  1
     991: CONST  0
     996: CONTAINS
     997: JUMPF  &"#temp::26::skip"
    1002: LOADL  2
    1007: COPY
    1008: INITL  4
    1013: JUMPF  &"#temp::29::compare"
    1018: POP
    1019: JUMP   &"#temp::26::skip"
"#temp::29::compare":
    1024: LOADL  4
    1029: VALEQ
    1030: JUMPF  &"#temp::27::cleanup"
"#temp::26::skip":
"#temp::30::assigned":
"#var::33::y::0x000056343246d0c0":
    1035: VAR
"#var::34::r::0x0000563432464920":
    1036: VAR
    1037: LOADL  1
    1042: CONST  1
    1047: CONTAINS
    1048: JUMPF  &"#temp::36::array_end"
    1053: LOADL  3
    1058: CLONE
    1059: COPY
    1060: CONST  0
    1065: ACCESS
    1066: COPY
    1067: INITL  5
    1072: JUMPF  &"#temp::37::compare"
    1077: POP
    1078: JUMP   &"#temp::38::assigned"
"#temp::37::compare":
    1083: LOADL  5
    1088: VALEQ
    1089: JUMPF  &"#temp::35::array_cleanup"
"#temp::38::assigned":
    1094: CONST  1
    1099: SKIP
    1100: COPY
    1101: INITL  6
    1106: JUMPF  &"#temp::39::compare"
    1111: POP
    1112: JUMP   &"#temp::40::assigned"
"#temp::39::compare":
    1117: LOADL  6
    1122: VALEQ
    1123: JUMPF  &"#temp::32::cleanup"
"#temp::40::assigned":
    1128: JUMP   &"#temp::36::array_end"
"#temp::35::array_cleanup":
    1133: POP
    1134: POP
    1135: JUMP   &"#temp::32::cleanup"
"#temp::36::array_end":
"#temp::31::skip":
    1140: CONST  true
    1145: CONST  false
    1150: CONS
    1151: CONST  unit
    1156: CONS
    1157: CLOSE  &"#temp::24::precall"
    1162: LOADL  7
    1167: CONST  false
    1172: SWAP
    1173: UNCONS
    1174: COPY
    1175: CONST  unit
    1180: VALEQ
    1181: SETL   8
    1186: CONST  false
    1191: VALNEQ
    1192: JUMPF  &"#temp::43::alt_second"
    1197: UNCONS
    1198: JUMPF  &"#temp::50::impl_outer"
"#temp::51::impl_inner":
    1203: CONST  false
    1208: SWAP
    1209: JUMPF  &"#temp::49::impl_cleans"
    1214: CONST  true
    1219: CONS
    1220: JUMP   &"#temp::47::impl_out"
"#temp::50::impl_outer":
    1225: CONST  false
    1230: SWAP
    1231: JUMPF  &"#temp::48::impl_cleanf"
    1236: LOADL  5
    1241: LOADL  1
    1246: CONST  4
    1251: CONTAINS
    1252: NOT
    1253: JUMP   &"#temp::52::skip"
    1258: UNSETL 4
"#temp::52::skip":
    1263: COPY
    1264: INITL  4
    1269: JUMPF  &"#temp::53::compare"
    1274: POP
    1275: JUMP   &"#temp::54::assigned"
"#temp::53::compare":
    1280: LOADL  4
    1285: VALEQ
    1286: JUMPF  &"#temp::48::impl_cleanf"
"#temp::54::assigned":
    1291: POP
    1292: CONST  true
    1297: JUMP   &"#temp::51::impl_inner"
"#temp::48::impl_cleanf":
    1302: CONST  false
    1307: CONS
    1308: JUMP   &"#temp::42::alt_maybe"
"#temp::49::impl_cleans":
    1313: CONST  true
    1318: CONS
    1319: JUMP   &"#temp::42::alt_maybe"
"#temp::47::impl_out":
    1324: CONST  true
    1329: CONS
    1330: JUMP   &"#temp::44::alt_out"
"#temp::43::alt_second":
    1335: UNCONS
    1336: JUMPF  &"#temp::55::setup"
"#temp::56::enter":
    1341: COPY
    1342: CALL   0
    1347: COPY
    1348: CONST  'done
    1353: VALNEQ
    1354: JUMPF  &"#temp::57::cleanup"
    1359: DESTRUCT
    1360: CONST  'next
    1365: VALEQ
    1366: JUMPF  &"#temp::57::cleanup"
    1371: UNCONS
    1372: LOADL  1
    1377: CONST  4
    1382: CONTAINS
    1383: NOT
    1384: JUMP   &"#temp::59::skip"
    1389: UNSETL 4
"#temp::59::skip":
    1394: COPY
    1395: INITL  4
    1400: JUMPF  &"#temp::60::compare"
    1405: POP
    1406: JUMP   &"#temp::61::assigned"
"#temp::60::compare":
    1411: LOADL  4
    1416: VALEQ
    1417: JUMPF  &"#temp::57::cleanup"
"#temp::61::assigned":
    1422: UNCONS
    1423: LOADL  1
    1428: CONST  6
    1433: CONTAINS
    1434: NOT
    1435: JUMP   &"#temp::62::skip"
    1440: UNSETL 6
"#temp::62::skip":
    1445: COPY
    1446: INITL  6
    1451: JUMPF  &"#temp::63::compare"
    1456: POP
    1457: JUMP   &"#temp::64::assigned"
"#temp::63::compare":
    1462: LOADL  6
    1467: VALEQ
    1468: JUMPF  &"#temp::57::cleanup"
"#temp::64::assigned":
    1473: POP
    1474: CONST  true
    1479: CONS
    1480: JUMP   &"#temp::58::end"
"#temp::57::cleanup":
    1485: POP
    1486: CONST  true
    1491: CONS
    1492: JUMP   &"#temp::46::alt_cleans"
"#temp::55::setup":
    1497: CONST  [||]
    1502: LOADL  1
    1507: CONST  4
    1512: CONTAINS
    1513: JUMPF  &"#temp::65::nope"
    1518: LOADL  4
    1523: LOADL  10
    1528: CONST  0
    1533: INSERT
    1534: POP
    1535: JUMP   &"#temp::66::next"
"#temp::65::nope":
    1540: CONST  unit
"#temp::66::next":
    1545: LOADL  1
    1550: CONST  6
    1555: CONTAINS
    1556: JUMPF  &"#temp::67::nope"
    1561: LOADL  6
    1566: LOADL  10
    1571: CONST  1
    1576: INSERT
    1577: POP
    1578: JUMP   &"#temp::68::next"
"#temp::67::nope":
    1583: CONST  unit
"#temp::68::next":
    1588: CALL   3
    1593: JUMP   &"#temp::56::enter"
"#temp::58::end":
    1598: CONST  false
    1603: CONS
    1604: JUMP   &"#temp::44::alt_out"
"#temp::42::alt_maybe":
    1609: LOADL  8
    1614: JUMPF  &"#temp::45::alt_cleanf"
    1619: POP
    1620: CONST  &"isel#56343246b550"
    1625: CALL   0
    1630: CONST  false
    1635: CONS
    1636: JUMP   &"#temp::43::alt_second"
"#temp::45::alt_cleanf":
    1641: CONST  true
    1646: CONS
    1647: SWAP
    1648: POP
    1649: JUMP   &"#temp::41::on_done"
"#temp::46::alt_cleans":
    1654: CONST  false
    1659: CONS
    1660: SWAP
    1661: POP
    1662: JUMP   &"#temp::41::on_done"
"#temp::44::alt_out":
    1667: SWAP
    1668: POP
    1669: SETL   7
    1674: LOADL  1
    1679: CONST  0
    1684: CONTAINS
    1685: JUMPF  &"#temp::69::eval"
    1690: LOADL  2
    1695: JUMP   &"#temp::70::next"
"#temp::69::eval":
    1700: LOADL  4
"#temp::70::next":
    1705: LOADL  1
    1710: CONST  1
    1715: CONTAINS
    1716: JUMPF  &"#temp::71::eval"
    1721: LOADL  3
    1726: JUMP   &"#temp::72::next"
"#temp::71::eval":
    1731: CONST  []
    1736: LOADL  5
    1741: INSERT
    1742: LOADL  6
    1747: GLUE
"#temp::72::next":
    1748: CONST  unit
    1753: SWAP
    1754: CONS
    1755: SWAP
    1756: CONS
    1757: CONST  'next
    1762: CONSTRUCT
    1763: RETURN
"#temp::41::on_done":
    1764: CONST  'done
    1769: RETURN
"#temp::32::cleanup":
    1770: POP
    1771: POP
"#temp::27::cleanup":
    1772: POP
    1773: JUMP   &"#temp::21::fail"
"#temp::23::end":
    1778: SWAP
    1779: CONST  1
    1784: CONS
    1785: SETL   0
    1790: RETURN
"#temp::21::fail":
    1791: CONST  (unit:2)
    1796: SETL   0
    1801: LOADL  0
    1806: UNCONS
"#temp::20::next_overload":
    1807: CONST  'done
    1812: RETURN
"main#563432468f10":
"trilogy:__entrypoint__":
    1813: CONST  &"isel#56343246b550"
    1818: CALL   0
    1823: CONST  false
    1828: CONS
    1829: UNCONS
    1830: JUMPF  &"#temp::75::setup"
"#temp::76::enter":
    1835: COPY
    1836: CALL   0
    1841: COPY
    1842: CONST  'done
    1847: VALNEQ
    1848: JUMPF  &"#temp::77::cleanup"
    1853: DESTRUCT
    1854: CONST  'next
    1859: VALEQ
    1860: JUMPF  &"#temp::77::cleanup"
    1865: UNCONS
    1866: CONST  1
    1871: VALEQ
    1872: JUMPF  &"#temp::77::cleanup"
    1877: UNCONS
    1878: CLONE
    1879: COPY
    1880: CONST  0
    1885: ACCESS
    1886: CONST  1
    1891: VALEQ
    1892: JUMPF  &"#temp::79::array_cleanup"
    1897: CONST  1
    1902: SKIP
    1903: COPY
    1904: CONST  0
    1909: ACCESS
    1910: CONST  2
    1915: VALEQ
    1916: JUMPF  &"#temp::79::array_cleanup"
    1921: CONST  1
    1926: SKIP
    1927: COPY
    1928: CONST  0
    1933: ACCESS
    1934: CONST  3
    1939: VALEQ
    1940: JUMPF  &"#temp::79::array_cleanup"
    1945: CONST  1
    1950: SKIP
    1951: POP
    1952: JUMP   &"#temp::80::array_end"
"#temp::79::array_cleanup":
    1957: POP
    1958: POP
    1959: JUMP   &"#temp::77::cleanup"
"#temp::80::array_end":
    1964: POP
    1965: CONST  true
    1970: CONS
    1971: JUMP   &"#temp::78::end"
"#temp::77::cleanup":
    1976: POP
    1977: CONST  true
    1982: CONS
    1983: JUMP   &"#temp::73::is_fail"
"#temp::75::setup":
    1988: CONST  [|0,1,|]
    1993: CONST  1
    1998: CONST  []
    2003: CONST  1
    2008: INSERT
    2009: CONST  2
    2014: INSERT
    2015: CONST  3
    2020: INSERT
    2021: CALL   3
    2026: JUMP   &"#temp::76::enter"
"#temp::78::end":
    2031: CONST  true
    2036: SLIDE  1
    2041: JUMP   &"#temp::74::is_end"
"#temp::73::is_fail":
    2046: CONST  false
    2051: SLIDE  1
"#temp::74::is_end":
    2056: POP
    2057: EXIT
    2058: CONST  unit
    2063: RETURN

