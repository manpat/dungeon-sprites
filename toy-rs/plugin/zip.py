import zipfile


# This is just for reference later - not used
def write_test_zip(fname):
	with zipfile.ZipFile(fname, 'w') as ar:
		with ar.open('eggs.txt', mode='w') as file:
			file.write(b"fooey")
		with ar.open('subdir/eggs.txt', mode='w') as file:
			file.write(b"fooey")
