import os
import shutil
import stat


def _onerror(func, path, exc_info):
    """
    Error handler for `shutil.rmtree` to fix it not being
    able to delete read-only files on Windows.

    Based on https://stackoverflow.com/a/2656405/20107
    """
    if not os.access(path, os.W_OK):
        os.chmod(path, stat.S_IWUSR)
        func(path)
    else:
        raise


def rm_rf(path: str):
    """A version of rmtree which actually works as expected on Windows
    """
    if not os.path.exists(path):
        return
    shutil.rmtree(path, onerror=_onerror)
