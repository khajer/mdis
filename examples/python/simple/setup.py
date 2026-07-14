#!/usr/bin/env python3

from setuptools import find_packages, setup

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="mdis-client",
    version="1.0.0",
    author="itsara konsombut",
    author_email="itsara@example.com",
    description="A Python client for MDIS (Multi-Device Integration Service)",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/example/mdis",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: System :: Networking",
    ],
    python_requires=">=3.6",
    install_requires=[
        # No external dependencies - all functionality uses Python's standard library
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "black>=21.0",
            "flake8>=3.8",
        ],
    },
    entry_points={
        "console_scripts": [
            "mdis-client=src.client:main",
        ],
    },
    keywords=[
        "mdis",
        "library",
        "python",
        "client",
        "tcp",
    ],
    project_urls={
        "Bug Reports": "https://github.com/example/mdis/issues",
        "Source": "https://github.com/example/mdis",
    },
)
