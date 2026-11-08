"""The source bootstrap must reject archive paths/links outside its pinned layout."""
import importlib.util
from pathlib import Path
import tarfile
import unittest

spec = importlib.util.spec_from_file_location('provision_git', Path(__file__).with_name('provision-git.py'))
provision = importlib.util.module_from_spec(spec)
spec.loader.exec_module(provision)


class ArchiveLayoutTests(unittest.TestCase):
    def member(self, name, kind=tarfile.REGTYPE, target=''):
        value = tarfile.TarInfo(name)
        value.type = kind
        value.linkname = target
        return value

    def test_regular_layout_and_exact_upstream_links(self):
        provision.validate_members([
            self.member('git-2.56.0', tarfile.DIRTYPE),
            self.member('git-2.56.0/Makefile'),
            self.member('git-2.56.0/RelNotes', tarfile.SYMTYPE,
                        'Documentation/RelNotes/2.56.0.adoc'),
            self.member('git-2.56.0/subprojects/git-gui', tarfile.SYMTYPE, '../git-gui'),
        ])

    def test_absolute_traversal_and_wrong_root(self):
        for path in ['/outside', '../outside', 'git-2.56.0/../../outside', 'other/Makefile']:
            with self.subTest(path=path), self.assertRaises(ValueError):
                provision.validate_members([self.member(path)])

    def test_unapproved_links_and_special_files(self):
        for member in [self.member('git-2.56.0/escape', tarfile.SYMTYPE, '/outside'),
                       self.member('git-2.56.0/RelNotes', tarfile.SYMTYPE, '../../outside'),
                       self.member('git-2.56.0/device', tarfile.CHRTYPE),
                       self.member('git-2.56.0/hardlink', tarfile.LNKTYPE, '/outside')]:
            with self.subTest(path=member.name), self.assertRaises(ValueError):
                provision.validate_members([member])

    def test_duplicate_paths_and_entries_through_known_links(self):
        for members in [[self.member('git-2.56.0/Makefile')]*2,
                        [self.member('git-2.56.0/subprojects/git-gui/escape')]]:
            with self.assertRaises(ValueError):
                provision.validate_members(members)


if __name__ == '__main__':
    unittest.main()
