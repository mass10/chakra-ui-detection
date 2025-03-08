default:

	# 確認
	file src/components/chakra/md5sum.txt

	find src/components/chakra -type f -ls | grep \.tsx > src/components/chakra/ls.txt.ci

	# 確認
	file src/components/chakra/md5sum.txt.ci

	# 確認
	cat src/components/chakra/md5sum.txt

	# 確認
	cat src/components/chakra/md5sum.txt.ci

	# 差異を確認
	diff -s -u src/components/chakra/md5sum.txt src/components/chakra/md5sum.txt.ci

chakra_checksum:
	cargo run > chakra_checksum-ci.txt

	cat chakra_checksum.txt
	cat chakra_checksum-ci.txt

	diff -s -u chakra_checksum-ci.txt chakra_checksum.txt
