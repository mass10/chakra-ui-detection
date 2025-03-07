default:

	# 確認
	file src/components/chakra/md5sum.txt

	find src/components/chakra -type f -ls | md5sum > src/components/chakra/md5sum.txt.ci

	# 確認
	file src/components/chakra/md5sum.txt.ci

	# 確認
	cat src/components/chakra/md5sum.txt

	# 確認
	cat src/components/chakra/md5sum.txt.ci

	# 差異を確認
	diff -s -u src/components/chakra/md5sum.txt src/components/chakra/md5sum.txt.ci
