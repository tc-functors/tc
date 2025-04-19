import boto3
import importlib.util
import io
import sys
import zipfile
from typing import Dict, Optional, List, Tuple
import re
from types import ModuleType


class LambdaImporter:
    """Class for importing packages from S3 in Lambda environments."""

    def __init__(self, bucket_name: str, prefix: str = "slab/"):
        """
        Initialize the Lambda importer.

        Args:
            bucket_name: Name of the S3 bucket containing the wheels
            prefix: Prefix path in the bucket where wheels are stored
        """
        self.bucket_name = bucket_name
        self.prefix = prefix
        self.s3_client = boto3.client('s3')
        self._loaded_modules = {}
        self._sys_path_added = False
        self._dependency_cache = {}  # Cache for package dependencies

    def _add_to_sys_path(self, wheel_dir: Dict[str, bytes]):
        """Add a new entry to sys.path for in-memory imports."""
        if not self._sys_path_added:
            # Create a unique identifier for this importer's path
            path_id = f"lambda_deps_from_s3_{id(self)}"
            if path_id not in sys.path:
                sys.path.insert(0, path_id)
            self._sys_path_added = True

    def _download_wheel_from_s3(self, s3_key: str) -> Optional[bytes]:
        """
        Download a wheel file from S3.

        Args:
            s3_key: S3 key of the wheel file

        Returns:
            Bytes content of the wheel file or None if download failed
        """
        try:
            response = self.s3_client.get_object(
                Bucket=self.bucket_name,
                Key=s3_key
            )
            return response['Body'].read()
        except Exception as e:
            print(f"Error downloading wheel {s3_key} from S3: {str(e)}")
            return None

    def _extract_wheel_in_memory(self, wheel_content: bytes) -> Dict[str, bytes]:
        """
        Extract wheel contents in memory.

        Args:
            wheel_content: Bytes content of the wheel file

        Returns:
            Dictionary mapping file paths to their contents
        """
        extracted_files = {}
        with zipfile.ZipFile(io.BytesIO(wheel_content)) as wheel_zip:
            for name in wheel_zip.namelist():
                if name.endswith('.py') or name.endswith('.pyd') or name.endswith('.so'):
                    extracted_files[name] = wheel_zip.read(name)
        return extracted_files

    def _get_package_metadata(self, wheel_content: bytes) -> Optional[Dict[str, str]]:
        """
        Extract metadata from wheel file.

        Args:
            wheel_content: Bytes content of the wheel file

        Returns:
            Dictionary of metadata fields or None if metadata not found
        """
        with zipfile.ZipFile(io.BytesIO(wheel_content)) as wheel_zip:
            metadata_files = [f for f in wheel_zip.namelist()
                            if f.endswith('METADATA') or f.endswith('PKG-INFO')]

            if not metadata_files:
                return None

            metadata_content = wheel_zip.read(metadata_files[0]).decode('utf-8')
            metadata = {}
            requires_dist = []

            for line in metadata_content.splitlines():
                if ':' in line:
                    key, value = line.split(':', 1)
                    key = key.strip()
                    value = value.strip()

                    if key == 'Requires-Dist':
                        requires_dist.append(value)
                    else:
                        metadata[key] = value

            if requires_dist:
                metadata['Requires-Dist'] = requires_dist

            return metadata

    def _parse_dependency_spec(self, spec: str) -> Tuple[str, Optional[str]]:
        """
        Parse a dependency specification into package name and version.

        Args:
            spec: Dependency specification string

        Returns:
            Tuple of (package_name, version_constraint)
        """
        # Remove environment markers
        if ';' in spec:
            spec = spec.split(';')[0].strip()

        # Handle version constraints
        if '(' in spec:
            pkg, version = spec.split('(', 1)
            version = version.rstrip(')').strip()
            return pkg.strip(), version
        elif '>=' in spec:
            pkg, version = spec.split('>=', 1)
            return pkg.strip(), '>=' + version.strip()
        elif '==' in spec:
            pkg, version = spec.split('==', 1)
            return pkg.strip(), '==' + version.strip()
        elif '<=' in spec:
            pkg, version = spec.split('<=', 1)
            return pkg.strip(), '<=' + version.strip()
        elif '>' in spec:
            pkg, version = spec.split('>', 1)
            return pkg.strip(), '>' + version.strip()
        elif '<' in spec:
            pkg, version = spec.split('<', 1)
            return pkg.strip(), '<' + version.strip()
        else:
            return spec.strip(), None

    def _get_package_dependencies(self, wheel_content: bytes) -> List[Tuple[str, Optional[str]]]:
        """
        Extract dependencies from wheel metadata.

        Args:
            wheel_content: Bytes content of the wheel file

        Returns:
            List of (package_name, version_constraint) tuples
        """
        metadata = self._get_package_metadata(wheel_content)
        if not metadata:
            return []

        dependencies = []
        requires_dist = metadata.get('Requires-Dist', [])
        if isinstance(requires_dist, str):
            requires_dist = [requires_dist]

        for dep_spec in requires_dist:
            pkg, version = self._parse_dependency_spec(dep_spec)
            if pkg:  # Skip empty package names
                dependencies.append((pkg, version))

        return dependencies

    def _create_in_memory_module(self, package_name: str, files: Dict[str, bytes]):
        """
        Create a module in memory from extracted files.

        Args:
            package_name: Name of the package
            files: Dictionary of file paths to their contents
        """
        # Create a module spec
        spec = importlib.util.spec_from_loader(
            package_name,
            loader=None,
            origin=f"lambda_deps_from_s3_{id(self)}"
        )

        # Create the module
        module = importlib.util.module_from_spec(spec)

        # Execute the module's code
        for path, content in files.items():
            if path.endswith('.py'):
                try:
                    # Compile and execute the code
                    code = compile(content, path, 'exec')
                    exec(code, module.__dict__)
                except Exception as e:
                    print(f"Error executing {path}: {str(e)}")

        return module

    def _find_latest_wheel(self, package_name: str) -> Optional[str]:
        """
        Find the latest wheel file for a package in S3.

        Args:
            package_name: Name of the package

        Returns:
            S3 key of the latest wheel or None if not found
        """
        try:
            response = self.s3_client.list_objects_v2(
                Bucket=self.bucket_name,
                Prefix=f"{self.prefix.rstrip('/')}/{package_name}"
            )

            if 'Contents' not in response or not response['Contents']:
                return None

            return max(response['Contents'], key=lambda x: x['LastModified'])['Key']
        except Exception as e:
            print(f"Error finding wheel for {package_name}: {str(e)}")
            return None

    def import_package(self, package_name: str, s3_key: Optional[str] = None) -> ModuleType:
        """
        Import a package from a wheel file in S3.

        Args:
            package_name: Name of the package to import
            s3_key: Optional explicit S3 key for the wheel file

        Returns:
            The imported module
        """
        # Check if already loaded in our cache
        if package_name in self._loaded_modules:
            return self._loaded_modules[package_name]

        try:
            # Try to import if already in Python's sys.modules
            if package_name != "torch_zip":
                module = importlib.import_module(package_name)
                self._loaded_modules[package_name] = module
                return module
        except ImportError:
            p

        try:
            if s3_key:
                wheel_content = self._download_wheel_from_s3(s3_key)
            else:
              s3_key = self._find_latest_wheel(package_name)
              if not s3_key:
                  raise ImportError(f"No wheels found for package {package_name}")
              wheel_content = self._download_wheel_from_s3(s3_key)

            # Extract the wheel and add to sys.path
            wheel_dir = self._extract_wheel_in_memory(wheel_content)
            self._add_to_sys_path(wheel_dir)

            # Import dependencies
            dependencies = self._get_package_dependencies(wheel_content)
            for dep_name, version in dependencies:
                try:
                    self.import_package(dep_name)
                except Exception as e:
                    print(f"Warning: Failed to import dependency {dep_name}: {e}")

            # Import the package and cache it
            module = importlib.import_module(package_name)
            self._loaded_modules[package_name] = module
            return module
        except Exception as e:
            print(f"Error importing package {package_name}: {e}")
            raise
