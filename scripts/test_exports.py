import pathlib
import tempfile
import unittest
import xml.etree.ElementTree as ET

import check_exports


# Independent expected coordinates from the original two-panel garment dimensions.
ORIGINAL_SVG = '''<svg xmlns="http://www.w3.org/2000/svg" width="2260mm" height="1560mm" viewBox="-530 -1560 2260 1560">
<path d="M -240 -1020 L 240 -1020 L 240 -1270 L 480 -1200 L 530 -1380 L 240 -1560 L 100 -1560 L 80 -1490 L -80 -1490 L -100 -1560 L -240 -1560 L -530 -1380 L -480 -1200 L -240 -1270 L -240 -1020 Z"/>
<path d="M 960 -1020 L 1440 -1020 L 1440 -1270 L 1680 -1200 L 1730 -1380 L 1440 -1560 L 1300 -1560 L 1280 -1490 L 1120 -1490 L 1100 -1560 L 960 -1560 L 670 -1380 L 720 -1200 L 960 -1270 L 960 -1020 Z"/>
</svg>'''


class SvgScaleTests(unittest.TestCase):
    def setUp(self):
        directory=tempfile.TemporaryDirectory(prefix='weftforge-svg-check-')
        self.addCleanup(directory.cleanup)
        self.path=pathlib.Path(directory.name)/'shirt.svg'
        self.path.write_text(ORIGINAL_SVG)

    def change(self, **attributes):
        tree=ET.parse(self.path)
        tree.getroot().attrib.update(attributes)
        tree.write(self.path,encoding='utf-8',xml_declaration=True)

    def test_original_fixture_with_origin_margin_is_one_to_one(self):
        self.assertEqual(check_exports.check_svg(self.path),2)

    def test_one_millimetre_page_cannot_pass_as_full_size_patterns(self):
        self.change(width='1mm',height='1mm')
        with self.assertRaisesRegex(AssertionError,'1:1'):
            check_exports.check_svg(self.path)

    def test_rescaled_viewbox_and_page_cannot_hide_wrong_pattern_scale(self):
        self.change(width='2.26mm',height='1.56mm',viewBox='-.53 -1.56 2.26 1.56')
        with self.assertRaisesRegex(AssertionError,'page bounds'):
            check_exports.check_svg(self.path)

    def test_pattern_coordinates_are_checked_separately_from_page_size(self):
        tree=ET.parse(self.path)
        path=tree.getroot().find('{http://www.w3.org/2000/svg}path')
        path.set('d',path.attrib['d'].replace('-530','-5300'))
        tree.write(self.path,encoding='utf-8',xml_declaration=True)
        with self.assertRaisesRegex(AssertionError,'pattern bounds'):
            check_exports.check_svg(self.path)

    def test_transform_cannot_silently_shrink_valid_coordinates(self):
        tree=ET.parse(self.path)
        tree.getroot().find('{http://www.w3.org/2000/svg}path').set('transform','scale(0.001)')
        tree.write(self.path,encoding='utf-8',xml_declaration=True)
        with self.assertRaisesRegex(AssertionError,'transform'):
            check_exports.check_svg(self.path)


if __name__=='__main__':
    unittest.main()
