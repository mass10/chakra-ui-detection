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
    echo "===== 既存のコンポーネント情報 ====="
	cat chakra_checksum.txt

    echo "===== CI 時実行時のコンポーネント情報 ====="
	cargo run > chakra_checksum-ci.txt
	cat chakra_checksum-ci.txt

    echo "===== 差分を出力 ====="
	diff -s -u chakra_checksum-ci.txt chakra_checksum.txt
