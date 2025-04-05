import boto3
import sys
import importlib.util
import io
import zipfile
import tempfile
import os
import json
import urllib.request
from typing import Optional, List, Dict
from pathlib import Path
from packaging import version
from packaging.specifiers import SpecifierSet

class S3WheelImporter:
    def __init__(self, bucket_name: str, prefix: str = "wheels/"):
        """
        Initialize the S3 wheel importer.

        Args:
            bucket_name: Name of the S3 bucket containing the wheels
            prefix: Prefix path in the bucket where wheels are stored
        """
        self.bucket_name = bucket_name
        self.prefix = prefix
        self.s3_client = boto3.client('s3')
        self._loaded_modules = {}
        self._temp_dir = None
        self._pypi_index = "https://pypi.org/simple"
        self._pypi_json = "https://pypi.org/pypi"

    def _ensure_temp_dir(self):
        """Ensure temporary directory exists for wheel extraction"""
        if self._temp_dir is None:
            self._temp_dir = tempfile.mkdtemp()
            # Add the temp directory to Python's path
            sys.path.insert(0, self._temp_dir)

    def _get_package_metadata(self, package_name: str) -> Dict:
        """Fetch package metadata from PyPI"""
        try:
            with urllib.request.urlopen(f"{self._pypi_json}/{package_name}/json") as response:
                return json.loads(response.read())
        except Exception as e:
            print(f"Error fetching metadata for {package_name}: {str(e)}")
            return {}

    def _get_compatible_wheel(self, package_name: str, version_str: str = None) -> Optional[str]:
        """
        Find the most compatible wheel for the current platform from PyPI.

        Args:
            package_name: Name of the package
            version_str: Version string (optional)

        Returns:
            URL of the compatible wheel or None if not found
        """
        metadata = self._get_package_metadata(package_name)
        if not metadata:
            return None

        releases = metadata.get('releases', {})
        if not releases:
            return None

        # Get available versions
        available_versions = [v for v in releases.keys() if v]
        if not available_versions:
            return None

        # Sort versions
        available_versions.sort(key=version.parse, reverse=True)

        # Select version
        target_version = version_str if version_str else available_versions[0]

        # Get release data for the version
        release_data = releases.get(target_version, [])
        if not release_data:
            return None

        # Get platform-specific wheel
        platform = self._detect_platform()
        python_version = f"cp{sys.version_info.major}{sys.version_info.minor}"

        # Find compatible wheel
        for file_info in release_data:
            if not file_info.get('filename', '').endswith('.whl'):
                continue

            filename = file_info['filename']
            if platform in filename and python_version in filename:
                return file_info['url']

        return None

    def _download_wheel(self, wheel_url: str) -> str:
        """
        Download a wheel file from URL and extract it to the temp directory.

        Args:
            wheel_url: URL of the wheel file

        Returns:
            Path to the extracted wheel directory
        """
        self._ensure_temp_dir()

        try:
            # Download the wheel
            with urllib.request.urlopen(wheel_url) as response:
                wheel_content = response.read()

            # Create a temporary file for the wheel
            wheel_path = os.path.join(self._temp_dir, os.path.basename(wheel_url))
            with open(wheel_path, 'wb') as f:
                f.write(wheel_content)

            # Extract the wheel
            with zipfile.ZipFile(wheel_path, 'r') as wheel_zip:
                wheel_zip.extractall(path=self._temp_dir)

            # Clean up the wheel file
            os.remove(wheel_path)

            return self._temp_dir

        except Exception as e:
            print(f"Error downloading wheel {wheel_url}: {str(e)}")
            return None

    def import_package(self, package_name: str, version: str = None) -> Optional[object]:
        """
        Import a package using its wheel from PyPI.

        Args:
            package_name: Name of the package to import
            version: Version of the package (optional)

        Returns:
            The imported module object
        """
        if package_name in self._loaded_modules:
            return self._loaded_modules[package_name]

        try:
            # Get compatible wheel URL
            wheel_url = self._get_compatible_wheel(package_name, version)
            if not wheel_url:
                raise ValueError(f"No compatible wheel found for package {package_name}")

            # Download and extract the wheel
            wheel_dir = self._download_wheel(wheel_url)
            if not wheel_dir:
                return None

            # Import the package
            module = importlib.import_module(package_name)
            self._loaded_modules[package_name] = module

            return module

        except Exception as e:
            print(f"Error importing package {package_name}: {str(e)}")
            return None

    def _detect_platform(self) -> str:
        """Detect the current platform for wheel selection"""
        import platform
        system = platform.system().lower()
        machine = platform.machine().lower()

        if system == 'linux':
            return f'manylinux2014_{machine}'
        elif system == 'darwin':
            return f'macosx_10_15_{machine}'
        elif system == 'windows':
            return f'win_{machine}'
        else:
            raise ValueError(f"Unsupported platform: {system}")
